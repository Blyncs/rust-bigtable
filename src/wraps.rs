use serde_json;
use protobuf::RepeatedField;
use request::BTRequest;
use data::{ReadModifyWriteRule, Mutation, Mutation_SetCell, Mutation_DeleteFromColumn};
use bigtable::MutateRowsRequest_Entry;
use utils::*;
use method::{BigTable, ReadRows, ReadModifyWriteRow, MutateRows, SampleRowKeys};
use error::BTErr;
use support::Table;
use goauth::auth::Token;
use std::collections::HashMap;
use serde_json::Value;


pub fn get_row_prefix(prefix: Option<&str>) -> String {
    match prefix {
        Some(x) => String::from(x),
        None => String::from("r")
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Row {
    pub row_key: Vec<u8>,
    pub family: Vec<u8>,
    pub qualifier: Vec<u8>,
    pub value: Vec<u8>
}

//impl Default for Row {
//    fn default() -> Self {
//        Row {
//            row_key: String::from("dummy_row_key"),
//            family: String::from("dummy_column_family"),
//            qualifier: String::from("dummy_column_qualifier"),
//            value: String::from("dummy_value"),
//        }
//    }
//}


/// ```
/// extern crate bigtable as bt;
///
/// use bt::utils::*;
/// use bt::error::BTErr;
/// use bt::wraps;
///
/// fn main() {
/// # #[allow(dead_code)]
/// # fn write_rows() -> Result<(), BTErr> {
///
///     let mut rows: Vec<wraps::Row> = vec!(wraps::Row::default()); // put some real data here
///     let token = get_auth_token("dummy_credentials_file_for_tests.json", true)?;
///     let table = Default::default();
///
///     let _ = wraps::bulk_write_rows(&mut rows, &token, table);
///
/// #    Ok(())
/// # }
/// }
/// ```
pub fn bulk_write_rows(rows: &Vec<Row>,
                       token: &Token,
                       table: Table) -> Result<String, BTErr> {
    let mut req = BTRequest {
        base: None,
        table: table,
        method: MutateRows::new()
    };

    let mut mutate_entries = Vec::new();

    for row in rows.iter() {
        let mut mutate_entry = MutateRowsRequest_Entry::new();
        mutate_entry.set_row_key(encode_vecu8(&row.row_key));

        let mut mutations: Vec<Mutation> = Vec::new();
        let mut mutation = Mutation::new();

        let set_cell = make_setcell_mutation(&row.qualifier, &row.family, &row.value);

        mutation.set_set_cell(set_cell);
        mutations.push(mutation);
        mutate_entry.set_mutations(RepeatedField::from_vec(mutations));
        mutate_entries.push(mutate_entry);
    }

    req.method.payload_mut().set_entries(RepeatedField::from_vec(mutate_entries));

    let response = req.execute(token)?;
    Ok(serde_json::to_string(&response)?)
}

pub fn bulk_mutate(
    mutations: HashMap<Vec<u8>, Vec<Mutation>>,
    token: &Token,
    table: Table
) -> Result<Value, BTErr> {
    let mut req = BTRequest {
        base: None,
        table: table,
        method: MutateRows::new()
    };

    let mut mutate_entries = Vec::new();

    for (row, mutations) in mutations {
        let mut mutate_entry = MutateRowsRequest_Entry::new();
        mutate_entry.set_row_key(encode_vecu8(&row));
        mutate_entry.set_mutations(RepeatedField::from_vec(mutations));
        mutate_entries.push(mutate_entry);
    }

    req.method.payload_mut().set_entries(RepeatedField::from_vec(mutate_entries));

    let response = req.execute(token)?;
    Ok(response)
}

/// ```
/// extern crate bigtable as bt;
///
/// use bt::utils::*;
/// use bt::error::BTErr;
/// use bt::wraps;
///
/// fn main() {
/// # #[allow(dead_code)]
/// # fn write_rows() -> Result<(), BTErr> {
///
///     let mut rows: Vec<wraps::Row> = vec!(wraps::Row::default()); // put some real data here
///     let token = get_auth_token("dummy_credentials_file_for_tests.json", true)?;
///     let table = Default::default();
///
///     let _ = wraps::write_rows(&mut rows, &token, table);
///
/// #    Ok(())
/// # }
/// }
/// ```
//pub fn write_rows(rows: &mut Vec<Row>,
//                  token: &Token,
//                  table: &Table) -> Result<usize, BTErr> {
//    let mut total = 0;
//
//    for row in rows.drain(..) {
//        let mut req = BTRequest {
//            base: None,
//            table: table.clone(),
//            method: ReadModifyWriteRow::new()
//        };
//
//        let mut rules: Vec<ReadModifyWriteRule> = Vec::new();
//
//        let rule = make_readmodifywrite_rule(&row.qualifier, &row.family, encode_str(&row.value));
//
//        rules.push(rule);
//
//        req.method.payload_mut().set_row_key(encode_str(&row.row_key));
//        req.method.payload_mut().set_rules(RepeatedField::from_vec(rules));
//
//        let _ = req.execute(token)?;
//        total += 1;
//    }
//    Ok(total)
//}

/// ```
/// extern crate bigtable as bt;
///
/// use bt::utils::*;
/// use bt::error::BTErr;
/// use bt::wraps;
///
/// fn main() {
/// # #[allow(dead_code)]
/// # fn read_rows(limit: i64) -> Result<(), BTErr> {
///
///    let token = get_auth_token("dummy_credentials_file_for_tests.json", true)?;
///    let table = Default::default();
///
///    let _ = wraps::read_rows(table, &token, Some(limit));
///
/// # Ok(())
/// # }
/// }
/// ```

use data::RowSet;

pub fn read_row(table: &Table, token: &Token, row: &Vec<u8>) -> Result<Value, BTErr> {
    let mut req = BTRequest {
        base: None,
        table: table.clone(),
        method: ReadRows::new()
    };

    let mut set = RowSet::new();
    set.set_row_keys(RepeatedField::from_vec(vec!(encode_vecu8(&row))));

    req.method.payload_mut().set_rows(set);

    let response = req.execute(token)?;
    Ok(response)
}

pub fn read_rows(table: &Table, token: &Token, rows: &Vec<Vec<u8>>) -> Result<Value, BTErr> {
    let mut req = BTRequest {
        base: None,
        table: table.clone(),
        method: ReadRows::new()
    };

    let mut set = RowSet::new();
    set.set_row_keys(RepeatedField::from_vec(rows.iter().map(|row| encode_vecu8(row)).collect()));

    req.method.payload_mut().set_rows(set);

    let response = req.execute(token)?;
    Ok(response)
}


pub fn make_setcell_mutation(column_qualifier: &Vec<u8>, column_family: &Vec<u8>, value: &Vec<u8>)
                             -> Mutation_SetCell {
    let mut set_cell = Mutation_SetCell::new();
    set_cell.set_column_qualifier(encode_vecu8(column_qualifier));
    set_cell.set_family_name(String::from_utf8(column_family.clone()).unwrap());
    set_cell.set_timestamp_micros(-1);
    set_cell.set_value(encode_vecu8(value));
    set_cell
}

pub fn make_delete_mutation(column_qualifier: &Option<Vec<u8>>, column_family: &Option<Vec<u8>>)
                            -> Mutation_DeleteFromColumn {
    let mut set_cell = Mutation_DeleteFromColumn::new();
    column_qualifier.as_ref().map(|column_qualifier| set_cell.set_column_qualifier(encode_vecu8(column_qualifier)));
    column_family.as_ref().map(|column_family| set_cell.set_family_name(String::from_utf8(column_family.clone()).unwrap()));
    set_cell
}

fn make_readmodifywrite_rule(column_qualifier: &str, column_familiy: &str, value: &Vec<u8>)
                             -> ReadModifyWriteRule {
    let mut rule = ReadModifyWriteRule::new();
    rule.set_family_name(String::from(column_familiy));
    rule.set_column_qualifier(encode_str(column_qualifier));
    rule.set_append_value(encode_vecu8(value));
    rule
}

fn sample_row_keys(token: &Token) -> Result<String, BTErr> {
    let req = BTRequest {
        base: None,
        table: Default::default(),
        method: SampleRowKeys::new()
    };
    let response = req.execute(token)?;
    Ok(serde_json::to_string(&response)?)
}
