use serde_json::{Map, Value};
use tracing::{error, info};

use crate::provider::SchemaProvider;

impl SchemaProvider {
    pub async fn get_multi_table_schema(&mut self, table_names: Vec<String>) -> String {
        let mut schemas = Vec::new();
        let self_clone = self.clone();
        for tb_name in table_names {
            match self.get_table_schema(&tb_name).await {
                Ok(schema) => {
                    schemas.push(self_clone._format_schema_for_llm(schema));
                }
                Err(e) => {
                    error!("\tFailed to get table schema: {e}");
                    schemas.push(format!("Failed to retrieving schema for {tb_name}: {e}"));
                }
            }
        }

        schemas.join("\n\n")
    }

    fn _format_schema_for_llm(&self, schema: Map<String, Value>) -> String {
        let table_name = schema
            .get("table_name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let columns = schema.get("columns").unwrap().as_array().unwrap();
        let foreign_key = schema.get("foreign").unwrap().as_array().unwrap();

        let mut col_lines = Vec::new();
        let mut fk_lines = Vec::new();

        for col in columns {
            let nullable = match col.get("nullable").unwrap().as_bool().unwrap() {
                true => "NULL".to_string(),
                false => "NOT NULL".to_string(),
            };

            let mut type_info = col.get("data_type").unwrap().as_str().unwrap().to_string();
            if let Some(max_len) = col.get("max_length") {
                type_info = format!("{} {}", type_info, max_len);
            }

            let default_info = match col.get("default") {
                Some(d) => format!("DEFAULT: {}", d.as_str().unwrap()),
                None => "".to_string(),
            };

            let name = col.get("name").unwrap().as_str().unwrap().to_string();

            let line = format!(
                "==[   {} - {} - {} - {}   ]==",
                name, type_info, nullable, default_info
            );

            col_lines.push(line);
        }

        for fk in foreign_key {
            let col = fk.get("column").unwrap().as_str().unwrap();
            let references = fk.get("references").unwrap().as_str().unwrap();

            fk_lines.push(format!(" {col} --> {references} "));
        }

        let lines = col_lines.join("\n");
        let mut _schema_text = String::new();
        _schema_text = format!("Table: {table_name}\n");
        _schema_text += format!("Columns:\n{lines}").as_str();

        if !fk_lines.is_empty() {
            _schema_text += format!("\n\nForeign Key: {}", fk_lines.join("\n\t")).as_str();
        }

        info!("\tFormat table information for LLM success!!!");
        _schema_text
    }
}

mod test {

    #[tokio::test]
    async fn test_get_multi_table() {
        use crate::provider::SchemaProvider;
        dotenv::from_path("../../.env").ok();
        let mut provider = SchemaProvider::default();
        let table_names = vec![
            String::from("public.users"),
            String::from("public.projects"),
            String::from("public.conversations"),
            String::from("public.roadmaps"),
            String::from("public.notes"),
            String::from("public.roadmap_phases"),
            String::from("public.milestones"),
            String::from("public.learning_resources"),
            String::from("public.messages"),
            String::from("public.task_progress"),
            String::from("public.tasks"),
        ];
        let multi_table = provider.get_multi_table_schema(table_names).await;

        println!("{multi_table}");
    }
}
