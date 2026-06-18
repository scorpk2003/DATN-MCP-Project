use serde_json::{Map, Value, json};
use tracing::{info, error};
use anyhow::Result;

use crate::provider::SchemaProvider;

impl SchemaProvider {
    pub async fn get_table_schema(&mut self, table_name: &String)
    -> Result<Map<String, Value>>
    {
        let mut schema_info = Map::new();
        let conn = match self.get_connections().await {
            Ok(connection) => {
                info!("\tGet connection for get table schema success!!!");
                connection
            },
            Err(e) => {
                tracing::error!("\tFailed to get connection for get table schema: {e}");
                return Err(e.into());
            }
        };

        let (schema_name, table_name) = match table_name.split_once(".") {
            Some((schema, table)) => (schema, table),
            None => ("public", table_name.as_str())
        };
        let column_query = "
                SELECT 
                    column_name,
                    data_type,
                    is_nullable,
                    column_default,
                    character_maximum_length,
                    ordinal_position
                FROM information_schema.columns 
                WHERE table_schema = $1 AND table_name = $2
                ORDER BY ordinal_position
        ";
        let columns = match conn.query(column_query, &[&schema_name, &table_name]).await {
            Ok(rows) => {
                info!("\tGet data infomation success!!!");
                rows
            },
            Err(e) => {
                error!("\tGet data information for `{schema_name}.{table_name} failed!!!!`");
                error!("\tError: {:?}", e);
                return Err(e.into());
            }
        };
        // Test get data
        // println!("{:?}", columns); --OK--

        let fk_key = "
        SELECT 
                    kcu.column_name,
                    ccu.table_schema AS foreign_table_schema,
                    ccu.table_name AS foreign_table_name,
                    ccu.column_name AS foreign_column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu 
                    ON tc.constraint_name = kcu.constraint_name
                JOIN information_schema.constraint_column_usage ccu 
                    ON ccu.constraint_name = tc.constraint_name
                WHERE tc.constraint_type = 'FOREIGN KEY' 
                    AND tc.table_schema = $1 
                    AND tc.table_name = $2
        ";
        let foreign_key = match conn.query(fk_key, &[&schema_name, &table_name]).await {
            Ok(fk) => {
                info!("\tGet foreign key for `{schema_name}.{table_name}` success!!!");
                fk
            },
            Err(e) => {
                error!("\tGet foreign key for `{schema_name}.{table_name}` failed!!!!");
                error!("\tError: {:?}", e);
                return Err(e.into());
            }
        };
        // Test get foreign key
        // println!("{:?}", foreign_key); --OK--

        let index_query = "
        SELECT 
                    indexname,
                    indexdef
                FROM pg_indexes 
                WHERE schemaname = $1 AND tablename = $2
        ";
        let indexes = match conn.query(index_query, &[&schema_name, &table_name]).await {
            Ok(idx) => {
                info!("\tGet index success!!!");
                idx
            },
            Err(e) => {
                error!("\tGet index for `{schema_name}.{table_name}` failed!!!");
                error!("\tError: {:?}", e);
                return Err(e.into());
            }
        };
        // Test Index
        // println!("{:?}", indexes); --OK--

        let mut schema_column = Vec::new();
        let mut schema_foreign = Vec::new();
        let mut schema_index = Vec::new();

        for column in columns {
            let name = column.get::<_, String>("column_name");
            let data_type = column.get::<_, String>("data_type");
            let nullable = column.get::<_, String>("is_nullable");
            let nullable = Value::from(nullable.contains("YES"));
            let default = column.get::<_, Option<String>>("column_default");
            let max_length = column.get::<_, Option<i32>>("character_maximum_length");
            let position = column.get::<_, i32>("ordinal_position");
            
            let col_result = json!({
                "name": name,
                "data_type": data_type,
                "nullable": nullable,
                "default": default.unwrap_or_default(),
                "max_length": max_length,
                "position": position,
            });
            schema_column.push(col_result);
        }

        for fk in foreign_key {
            let column = fk.get::<_, String>("column_name");

            let f_table_schema = fk.get::<_, String>("foreign_table_schema");
            let f_table_name = fk.get::<_, String>("foreign_table_name");
            let f_column_name = fk.get::<_, String>("foreign_column_name");
            let references = Value::String(format!("{f_table_schema}.{f_table_name}.{f_column_name}"));

            let column = Value::from(column);
            let fk_result = json!({
                "column": column,
                "references": references,
            });
            schema_foreign.push(fk_result);
        }

        for idx in indexes {
            let name = idx.get::<_, String>("indexname");
            let defination = idx.get::<_, String>("indexdef");

            let idx_result = json!({
                "name": name,
                "defination": defination,
            });

            schema_index.push(idx_result);
        }

        let tb_name = format!("{schema_name}.{table_name}");
        let columns = Value::Array(schema_column);
        let foreign = Value::Array(schema_foreign);
        let indexes = Value::Array(schema_index);

        schema_info.insert("table_name".to_string(), Value::String(tb_name));
        schema_info.insert("columns".to_string(), columns);
        schema_info.insert("foreign".to_string(), foreign);
        schema_info.insert("indexes".to_string(), indexes);
        Ok(schema_info)
    }
}

mod test {
    
    #[tokio::test]
    async fn test_get_table_schema() {
        use crate::provider::SchemaProvider;
        dotenv::from_path("../../.env").ok();
        let mut provider = SchemaProvider::default();
        let table = provider.get_table_schema(&"public.learning_resources".to_string()).await;
        println!("=={:?}==",table);
    }
}