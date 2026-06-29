use std::{collections::HashMap, env};

use reqwest::{Client, StatusCode};
use resource_service::{
    AppError, AppResult,
    corpus::{OfficialSource, load_validated_manifests, validate_seed_url},
    models::{
        ApiEnvelope, CrawlSeed, CrawlSeedRequest, Page, PageQuery, SourcePatchRequest,
        SourceRequest, SourceSite,
    },
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
struct Args {
    base_url: String,
    source_catalog: String,
    seed_manifest: String,
    topic_manifest: String,
    dry_run: bool,
    verify_urls: bool,
}

#[derive(Debug, Default, Serialize)]
struct SeedSummary {
    #[serde(rename = "sourcesCreated")]
    sources_created: usize,
    #[serde(rename = "sourcesUpdated")]
    sources_updated: usize,
    #[serde(rename = "seedsCreated")]
    seeds_created: usize,
    #[serde(rename = "seedsUpdated")]
    seeds_updated: usize,
    skipped: usize,
    invalid: Vec<InvalidSeed>,
}

#[derive(Debug, Serialize)]
struct InvalidSeed {
    #[serde(rename = "seedId")]
    seed_id: String,
    url: String,
    status: String,
    reason: String,
}

fn load_env() {
    dotenv::dotenv().ok();
    for path in ["../.env", "../../.env", "../../../.env"] {
        dotenv::from_path(path).ok();
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    load_env();

    let args = Args::parse()?;
    let (catalog, seeds, topics) = load_validated_manifests(
        &args.source_catalog,
        &args.seed_manifest,
        &args.topic_manifest,
    )?;
    let client = Client::builder()
        .user_agent("self-learn-resource-worker/0.2.1")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|err| AppError::Internal(err.to_string()))?;
    let mut summary = SeedSummary::default();
    let mut source_map = existing_sources(&client, &args.base_url).await?;

    for source in &catalog.sources {
        if let Some(existing) = source_map.get(&source.base_url).cloned() {
            if !args.dry_run {
                patch_source(&client, &args.base_url, &existing, source).await?;
            }
            summary.sources_updated += 1;
        } else {
            if !args.dry_run {
                let created = create_source(&client, &args.base_url, source).await?;
                source_map.insert(source.base_url.clone(), created);
            }
            summary.sources_created += 1;
        }
    }

    let existing_seed_keys = existing_seeds(&client, &args.base_url).await?;
    let sources_by_id = catalog
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<HashMap<_, _>>();
    let topic_aliases = topics
        .topics
        .iter()
        .map(|topic| (topic.topic_id.as_str(), topic.aliases.clone()))
        .collect::<HashMap<_, _>>();

    for seed in seeds.seeds {
        let source = sources_by_id
            .get(seed.source_id.as_str())
            .expect("seed manifest was validated");
        let source_site = source_map
            .get(&source.base_url)
            .expect("source was created or loaded");
        let key = format!("{}|{}|{}", source_site.id, seed.seed_type, seed.url);
        if existing_seed_keys.contains_key(&key) {
            summary.seeds_updated += 1;
            continue;
        }

        let mut enabled = seed.enabled;
        if args.verify_urls {
            match verify_url(&client, source, &seed.url).await {
                Ok(final_url) => {
                    validate_seed_url(source, &final_url)?;
                }
                Err(reason) => {
                    enabled = false;
                    summary.invalid.push(InvalidSeed {
                        seed_id: seed.seed_id.clone(),
                        url: seed.url.clone(),
                        status: "invalid".to_string(),
                        reason,
                    });
                }
            }
        }

        if args.dry_run {
            summary.skipped += 1;
            continue;
        }

        let aliases = seed
            .topic_ids
            .iter()
            .flat_map(|topic_id| {
                topic_aliases
                    .get(topic_id.as_str())
                    .cloned()
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        create_seed(
            &client,
            &args.base_url,
            CrawlSeedRequest {
                source_site_id: Some(source_site.id),
                seed_url: seed.url.clone(),
                seed_type: Some(seed.seed_type.clone()),
                max_depth: Some(seed.max_depth),
                priority: Some(seed.priority),
                enabled: Some(enabled),
                metadata: Some(json!({
                    "officialSeedId": seed.seed_id,
                    "topicIds": seed.topic_ids,
                    "topicAliases": aliases,
                    "expectedResourceKind": seed.expected_resource_kind,
                    "notes": seed.notes,
                    "maxDepth": seed.max_depth,
                    "finalUrlVerified": args.verify_urls,
                })),
            },
        )
        .await?;
        summary.seeds_created += 1;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&summary).expect("summary serializes")
    );
    Ok(())
}

impl Args {
    fn parse() -> AppResult<Self> {
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            base_url: "http://127.0.0.1:3200".to_string(),
            source_catalog: "config/official_source_catalog.yaml".to_string(),
            seed_manifest: "config/official_topic_seed_manifest.yaml".to_string(),
            topic_manifest: "config/evaluation_topic_manifest.yaml".to_string(),
            dry_run: true,
            verify_urls: true,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--base-url" => parsed.base_url = next_value(&mut args, "--base-url")?,
                "--source-catalog" => {
                    parsed.source_catalog = next_value(&mut args, "--source-catalog")?
                }
                "--seed-manifest" => {
                    parsed.seed_manifest = next_value(&mut args, "--seed-manifest")?
                }
                "--topic-manifest" => {
                    parsed.topic_manifest = next_value(&mut args, "--topic-manifest")?
                }
                "--dry-run" => parsed.dry_run = parse_bool(&next_value(&mut args, "--dry-run")?),
                "--verify-urls" => {
                    parsed.verify_urls = parse_bool(&next_value(&mut args, "--verify-urls")?)
                }
                other => return Err(AppError::Validation(format!("unknown argument {other}"))),
            }
        }
        Ok(parsed)
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> AppResult<String> {
    args.next()
        .ok_or_else(|| AppError::Validation(format!("{name} requires a value")))
}

fn parse_bool(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes")
}

async fn existing_sources(
    client: &Client,
    base_url: &str,
) -> AppResult<HashMap<String, SourceSite>> {
    let envelope: ApiEnvelope<Page<SourceSite>> = client
        .get(format!("{base_url}/sources?limit=100"))
        .send()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?
        .json()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(envelope
        .data
        .unwrap_or(Page {
            items: Vec::new(),
            pagination: resource_service::models::PaginationMeta {
                limit: PageQuery::default_limit(),
                offset: 0,
                total: 0,
                has_more: false,
            },
        })
        .items
        .into_iter()
        .map(|source| (source.base_url.clone(), source))
        .collect())
}

async fn existing_seeds(client: &Client, base_url: &str) -> AppResult<HashMap<String, CrawlSeed>> {
    let envelope: ApiEnvelope<Page<CrawlSeed>> = client
        .get(format!("{base_url}/crawl/seeds?limit=100"))
        .send()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?
        .json()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(envelope
        .data
        .map(|page| {
            page.items
                .into_iter()
                .filter_map(|seed| {
                    seed.source_id.map(|source_id| {
                        (
                            format!("{source_id}|{}|{}", seed.kind, seed.seed_value),
                            seed,
                        )
                    })
                })
                .collect()
        })
        .unwrap_or_default())
}

async fn create_source(
    client: &Client,
    base_url: &str,
    source: &OfficialSource,
) -> AppResult<SourceSite> {
    let request = SourceRequest {
        name: source.name.clone(),
        kind: Some(source.kind.clone()),
        base_url: source.base_url.clone(),
        trust_tier: Some(source.trust_tier),
        language_hint: Some(source.language_hint.clone()),
        enabled: Some(source.enabled),
        is_official: Some(source.is_official),
        crawl_policy: Some(
            serde_json::to_value(&source.crawl_policy).unwrap_or_else(|_| json!({})),
        ),
        allowed_paths: Some(source.allowed_paths.clone()),
        blocked_paths: Some(source.blocked_paths.clone()),
        tags: Some(vec![
            source.id.clone(),
            "official_corpus_v0_2_1".to_string(),
        ]),
        notes: Some(format!("official source catalog id: {}", source.id)),
    };
    post_envelope(client, &format!("{base_url}/sources"), &request).await
}

async fn patch_source(
    client: &Client,
    base_url: &str,
    existing: &SourceSite,
    source: &OfficialSource,
) -> AppResult<SourceSite> {
    let request = SourcePatchRequest {
        name: Some(source.name.clone()),
        enabled: Some(source.enabled),
        crawl_policy: Some(
            serde_json::to_value(&source.crawl_policy).unwrap_or_else(|_| json!({})),
        ),
        allowed_paths: Some(source.allowed_paths.clone()),
        blocked_paths: Some(source.blocked_paths.clone()),
        notes: Some(format!("official source catalog id: {}", source.id)),
    };
    let envelope: ApiEnvelope<SourceSite> = client
        .patch(format!("{base_url}/sources/{}", existing.id))
        .json(&request)
        .send()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?
        .json()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    envelope
        .data
        .ok_or_else(|| AppError::Internal("missing source patch response".to_string()))
}

async fn create_seed(
    client: &Client,
    base_url: &str,
    request: CrawlSeedRequest,
) -> AppResult<CrawlSeed> {
    post_envelope(client, &format!("{base_url}/crawl/seeds"), &request).await
}

async fn post_envelope<T, B>(client: &Client, url: &str, body: &B) -> AppResult<T>
where
    T: for<'de> serde::Deserialize<'de>,
    B: Serialize + ?Sized,
{
    let envelope: ApiEnvelope<T> = client
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?
        .json()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    envelope
        .data
        .ok_or_else(|| AppError::Internal(format!("missing response data for {url}")))
}

async fn verify_url(client: &Client, source: &OfficialSource, url: &str) -> Result<String, String> {
    let response = match client.head(url).send().await {
        Ok(response) if response.status() == StatusCode::METHOD_NOT_ALLOWED => {
            client.get(url).send().await
        }
        other => other,
    }
    .map_err(|err| err.to_string())?;
    if response.status() != StatusCode::OK {
        return Err(format!("HTTP_{}", response.status().as_u16()));
    }
    let final_url = response.url().to_string();
    validate_seed_url(source, &final_url).map_err(|err| err.to_string())?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !content_type.is_empty()
        && !content_type.contains("text/html")
        && !content_type.contains("text/plain")
        && !content_type.contains("text/markdown")
        && !content_type.contains("application/pdf")
    {
        return Err(format!("UNSUPPORTED_CONTENT_TYPE:{content_type}"));
    }
    Ok(final_url)
}

trait PageQueryDefault {
    fn default_limit() -> i64;
}

impl PageQueryDefault for PageQuery {
    fn default_limit() -> i64 {
        100
    }
}
