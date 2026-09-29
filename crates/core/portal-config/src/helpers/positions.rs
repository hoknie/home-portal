use toml_edit::{DocumentMut, Item, Table};

pub fn next_position(document: &DocumentMut) -> isize {
    highest(document.as_item()) + 1
}

pub fn highest(item: &Item) -> isize {
    let mut highest = 0;
    visit_tables(item, &mut |table| {
        highest = highest.max(table.position().unwrap_or(0));
    });
    highest
}

pub fn shifted(mut item: Item, next: isize) -> Item {
    let mut lowest = isize::MAX;
    visit_tables(&item, &mut |table| {
        if let Some(position) = table.position() {
            lowest = lowest.min(position);
        }
    });
    let offset = if lowest == isize::MAX {
        next
    } else {
        next - lowest
    };
    visit_tables_mut(&mut item, &mut |table| {
        if let Some(position) = table.position() {
            table.set_position(Some(position + offset));
        }
    });
    item
}

fn visit_tables(item: &Item, visit: &mut dyn FnMut(&Table)) {
    match item {
        Item::Table(table) => {
            visit(table);
            for (_, child) in table.iter() {
                visit_tables(child, visit);
            }
        }
        Item::ArrayOfTables(entries) => {
            for table in entries.iter() {
                visit(table);
                for (_, child) in table.iter() {
                    visit_tables(child, visit);
                }
            }
        }
        _ => {}
    }
}

fn visit_tables_mut(item: &mut Item, visit: &mut dyn FnMut(&mut Table)) {
    match item {
        Item::Table(table) => {
            visit(table);
            for (_, child) in table.iter_mut() {
                visit_tables_mut(child, visit);
            }
        }
        Item::ArrayOfTables(entries) => {
            for table in entries.iter_mut() {
                visit(table);
                for (_, child) in table.iter_mut() {
                    visit_tables_mut(child, visit);
                }
            }
        }
        _ => {}
    }
}
