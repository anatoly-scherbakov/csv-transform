use std::io;
use csv::{ByteRecord, Writer, ReaderBuilder};

use crate::config::create_transformer;
use crate::transformer::{Transformer, Expression};
use crate::options::{Options, Variables};
use crate::printable_error::ConfigParseError;


fn apply_column(
    column: &Vec<Expression>,
    record: &ByteRecord,
    variables: &Variables,
) -> String {
    let mut value: Option<String> = None;

    for expression in column.iter() {
        value = expression.apply(value, record, variables);
    }

    match value {
        Some(content) => content,
        None => String::new(),
    }
}


fn transform(
    record: ByteRecord,
    transformer: &Transformer,
    variables: &Variables,
) -> ByteRecord {
    let output: Vec<String> = transformer.columns.iter().map(
        |column| apply_column(
            column,
            &record,
            &variables,
        )
    ).collect();

    ByteRecord::from(output)
}


pub fn process(options: Options) -> Result<(), String> {
    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .from_reader(io::stdin());

    let mut writer = Writer::from_writer(io::stdout());

    let headers = reader.headers().unwrap().clone();

    let maybe_transformer = create_transformer(
        &options.config,
        &headers,
    );

    if let Err(err) = maybe_transformer {
        return Err(err.error_description);
    }

    let transformer = maybe_transformer.unwrap();

    writer.write_record(&transformer.headers).unwrap();

    for result in reader.byte_records() {
        let record = result.unwrap();

        writer.write_record(&transform(
            record,
            &transformer,
            &options.variables,
        )).unwrap();
    }

    Ok(writer.flush().unwrap())
}
