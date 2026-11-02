use std::{
    fs::File,
    path::Path,
    sync::Arc,
};

use anyhow::{Context, Result};
use arrow::{
    array::{
        ArrayRef,
        FixedSizeBinaryArray,
        Int64Array,
        StringArray,
        UInt64Array,
    },
    datatypes::{
        DataType,
        Field,
        Schema,
        SchemaRef,
    },
    record_batch::RecordBatch,
};
use parquet::arrow::ArrowWriter;
use uuid::Uuid;

use aispy::event::Event;

pub struct EventWriter {
    run_id: Uuid,
    next_event_id: u64,
    schema: SchemaRef,
    writer: ArrowWriter<File>,
}

impl EventWriter {
    pub fn create(
        path: impl AsRef<Path>,
        run_id: Uuid,
    ) -> Result<Self> {
        let schema = events_schema();

        let file = File::create(path.as_ref()).with_context(|| {
            format!(
                "failed to create {}",
                path.as_ref().display()
            )
        })?;

        let writer =
            ArrowWriter::try_new(file, schema.clone(), None)
                .context("failed to create Parquet writer")?;

        Ok(Self {
            run_id,
            next_event_id: 0,
            schema,
            writer,
        })
    }

    pub fn write(
        &mut self,
        events: &[Event],
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let batch = self.record_batch(events)?;

        self.writer
            .write(&batch)
            .context("failed to write events batch")?;

        self.next_event_id += events.len() as u64;

        Ok(())
    }

    pub fn close(self) -> Result<()> {
        self.writer
            .close()
            .context("failed to close events.parquet")?;

        Ok(())
    }

    fn record_batch(
        &self,
        events: &[Event],
    ) -> Result<RecordBatch> {
        let event_ids =
            self.next_event_id..self.next_event_id + events.len() as u64;

        let run_id = *self.run_id.as_bytes();

        let run_ids =
            FixedSizeBinaryArray::try_from_iter(
                std::iter::repeat_n(
                    run_id,
                    events.len(),
                ),
            )
            .context("failed to construct run_id column")?;

        let event_ids =
            UInt64Array::from_iter_values(event_ids);

        let timestamps =
            UInt64Array::from_iter_values(
                events.iter().map(|event| event.timestamp),
            );

        let clock_domains =
            StringArray::from_iter_values(
                events
                    .iter()
                    .map(|event| event.clock_domain.as_str()),
            );

        let sources =
            StringArray::from_iter_values(
                events
                    .iter()
                    .map(|event| event.source.as_str()),
            );

        let kinds =
            StringArray::from_iter_values(
                events
                    .iter()
                    .map(|event| event.kind.as_str()),
            );

        let instance_ids =
            UInt64Array::from_iter(
                events.iter().map(|event| event.instance_id),
            );

        let entity_ids =
            UInt64Array::from_iter(
                events.iter().map(|event| event.entity_id),
            );

        let thread_entity_ids =
            UInt64Array::from_iter(
                events
                    .iter()
                    .map(|event| event.thread_entity_id),
            );

        let related_entity_ids =
            UInt64Array::from_iter(
                events
                    .iter()
                    .map(|event| event.related_entity_id),
            );

        let values =
            Int64Array::from_iter(
                events.iter().map(|event| event.value_i64),
            );

        let columns: Vec<ArrayRef> = vec![
            Arc::new(run_ids),
            Arc::new(event_ids),
            Arc::new(timestamps),
            Arc::new(clock_domains),
            Arc::new(sources),
            Arc::new(kinds),
            Arc::new(instance_ids),
            Arc::new(entity_ids),
            Arc::new(thread_entity_ids),
            Arc::new(related_entity_ids),
            Arc::new(values),
        ];

        RecordBatch::try_new(
            self.schema.clone(),
            columns,
        )
        .context("failed to construct events RecordBatch")
    }
}

fn events_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new(
            "run_id",
            DataType::FixedSizeBinary(16),
            false,
        ),
        Field::new(
            "event_id",
            DataType::UInt64,
            false,
        ),
        Field::new(
            "timestamp",
            DataType::UInt64,
            false,
        ),
        Field::new(
            "clock_domain",
            DataType::Utf8,
            false,
        ),
        Field::new(
            "source",
            DataType::Utf8,
            false,
        ),
        Field::new(
            "kind",
            DataType::Utf8,
            false,
        ),
        Field::new(
            "instance_id",
            DataType::UInt64,
            true,
        ),
        Field::new(
            "entity_id",
            DataType::UInt64,
            true,
        ),
        Field::new(
            "thread_entity_id",
            DataType::UInt64,
            true,
        ),
        Field::new(
            "related_entity_id",
            DataType::UInt64,
            true,
        ),
        Field::new(
            "value_i64",
            DataType::Int64,
            true,
        ),
    ]))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use arrow::array::Array;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use tempfile::tempdir;
    use uuid::Uuid;

    use aispy::event::{
        ClockDomain,
        Event,
        EventKind,
        EventSource,
    };

    use super::EventWriter;

    #[test]
    fn writes_events_parquet() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("events.parquet");

        let run_id = Uuid::now_v7();

        let events = vec![
            Event {
                timestamp: 1_000,
                clock_domain: ClockDomain::HostMonotonic,
                source: EventSource::Ebpf,
                kind: EventKind::SchedulerWakeup,
                instance_id: None,
                entity_id: Some(1),
                thread_entity_id: Some(2),
                related_entity_id: Some(3),
                value_i64: None,
            },
            Event {
                timestamp: 2_000,
                clock_domain: ClockDomain::HostMonotonic,
                source: EventSource::Semantic,
                kind: EventKind::ObservationBegin,
                instance_id: Some(42),
                entity_id: None,
                thread_entity_id: Some(2),
                related_entity_id: None,
                value_i64: None,
            },
        ];

        let mut writer =
            EventWriter::create(&path, run_id).unwrap();

        writer.write(&events).unwrap();
        writer.close().unwrap();

        let file = File::open(path).unwrap();

        let mut reader =
            ParquetRecordBatchReaderBuilder::try_new(file)
                .unwrap()
                .build()
                .unwrap();

        let batch =
            reader.next().unwrap().unwrap();

        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 11);

        let run_ids = batch.column(0);

        assert_eq!(run_ids.len(), 2);

        assert!(reader.next().is_none());
    }
}
