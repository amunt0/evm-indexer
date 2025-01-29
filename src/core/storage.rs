use crate::config::Config;
use crate::models::Block;
use anyhow::Result;
use arrow::{
    array::{UInt64Builder, StringBuilder, ListBuilder, StructBuilder, ArrayBuilder},
    datatypes::{Schema, Field, DataType, Fields},
    record_batch::RecordBatch,
};
use parquet::{
    arrow::ArrowWriter,
    file::properties::WriterProperties,
};
use std::{fs::File, sync::Arc, path::PathBuf};
use chrono::Utc;

pub struct StorageManager {
    data_dir: PathBuf,
    current_batch: Vec<Block>,
    batch_size: usize,
    schema: Arc<Schema>,
    current_writer: Option<ArrowWriter<File>>,
}

impl StorageManager {
    pub fn new(config: &Config) -> Result<Self> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("number", DataType::UInt64, false),
            Field::new("hash", DataType::Utf8, false),
            Field::new("timestamp", DataType::UInt64, false),
            Field::new("transactions", DataType::List(Arc::new(Field::new(
                "transaction",
                DataType::Struct(Fields::from(vec![
                    Field::new("hash", DataType::Utf8, false),
                    Field::new("from", DataType::Utf8, false),
                    Field::new("to", DataType::Utf8, true),
                    Field::new("value", DataType::Utf8, false),
                ])),
                false,
            ))), false),
        ]));

        std::fs::create_dir_all(&config.data_dir)?;

        Ok(Self {
            data_dir: config.data_dir.clone(),
            current_batch: Vec::with_capacity(config.blocks_in_memory),
            batch_size: config.blocks_in_memory,
            schema,
            current_writer: None,
        })
    }

    fn create_new_file(&mut self) -> Result<()> {
        let timestamp = Utc::now();
        let filename = format!("blocks_{}.parquet", timestamp.format("%Y%m%d_%H%M%S"));
        let path = self.data_dir.join(filename);
        
        let file = File::create(path)?;
        let props = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .build();

        self.current_writer = Some(ArrowWriter::try_new(
            file,
            self.schema.clone(),
            Some(props),
        )?);

        Ok(())
    }

    pub async fn store_block(&mut self, block: Block) -> Result<()> {
        self.current_batch.push(block);

        if self.current_batch.len() >= self.batch_size {
            self.flush_batch()?;
        }

        Ok(())
    }

    fn flush_batch(&mut self) -> Result<()> {
        if self.current_batch.is_empty() {
            return Ok(());
        }

        let data_len: usize = self.current_batch.iter()
            .map(|block| block.transactions.len())
            .sum();

        let mut number_builder = UInt64Builder::with_capacity(self.current_batch.len());
        let mut hash_builder = StringBuilder::with_capacity(self.current_batch.len(), self.current_batch.len() * 66);
        let mut timestamp_builder = UInt64Builder::with_capacity(self.current_batch.len());

        // Create builders for the transaction struct
        let mut tx_hash_builder = StringBuilder::with_capacity(data_len, data_len * 66);
        let mut tx_from_builder = StringBuilder::with_capacity(data_len, data_len * 42);
        let mut tx_to_builder = StringBuilder::with_capacity(data_len, data_len * 42);
        let mut tx_value_builder = StringBuilder::with_capacity(data_len, data_len * 32);

        let tx_struct_builder = StructBuilder::new(
            Fields::from(vec![
                Field::new("hash", DataType::Utf8, false),
                Field::new("from", DataType::Utf8, false),
                Field::new("to", DataType::Utf8, true),
                Field::new("value", DataType::Utf8, false),
            ]),
            vec![
                Box::new(tx_hash_builder),
                Box::new(tx_from_builder),
                Box::new(tx_to_builder),
                Box::new(tx_value_builder),
            ],
        );

        let mut tx_list_builder = ListBuilder::new(tx_struct_builder);

        for block in &self.current_batch {
            number_builder.append_value(block.number);
            hash_builder.append_value(&block.hash);
            timestamp_builder.append_value(block.timestamp);
            
            let tx_list_values = tx_list_builder.values();
            if let Some(struct_builder) = tx_list_values.as_any_mut().downcast_mut::<StructBuilder>() {
                for tx in &block.transactions {
                    if let Some(builder) = struct_builder.field_builder::<StringBuilder>(0) {
                        builder.append_value(&tx.hash);
                    }
                    if let Some(builder) = struct_builder.field_builder::<StringBuilder>(1) {
                        builder.append_value(&tx.from);
                    }
                    if let Some(builder) = struct_builder.field_builder::<StringBuilder>(2) {
                        if let Some(to) = &tx.to {
                            builder.append_value(to);
                        } else {
                            builder.append_null();
                        }
                    }
                    if let Some(builder) = struct_builder.field_builder::<StringBuilder>(3) {
                        builder.append_value(&tx.value);
                    }
                }
            }
            tx_list_builder.append(true);
        }

        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(number_builder.finish()),
                Arc::new(hash_builder.finish()),
                Arc::new(timestamp_builder.finish()),
                Arc::new(tx_list_builder.finish()),
            ],
        )?;

        if self.current_writer.is_none() {
            self.create_new_file()?;
        }

        if let Some(writer) = &mut self.current_writer {
            writer.write(&batch)?;
        }

        self.current_batch.clear();
        Ok(())
    }

    pub async fn rotate_file(&mut self) -> Result<()> {
        self.flush_batch()?;

        if let Some(writer) = self.current_writer.take() {
            writer.close()?;
        }

        self.create_new_file()?;
        Ok(())
    }
}