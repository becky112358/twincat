use std::ops::RangeInclusive;

use super::{DataType, Symbol, SymbolsAndDataTypes};

impl SymbolsAndDataTypes {
    pub fn persistent(&self) -> Vec<String> {
        let mut output = Vec::new();
        let filter = Filter {
            filter: &|symbol| symbol.persistent,
        };
        for symbol in self.symbols.0.iter() {
            if symbol.1.persistent {
                output.push(symbol.0.to_string());
            } else {
                let data_type = if let Some(dt) = self.data_types.0.get(&symbol.1.data_type_name) {
                    dt
                } else {
                    continue;
                };
                let persistents = self.recurse(&filter, data_type);
                for p0 in persistents {
                    output.extend(p0.to_strings(symbol.0.to_string()));
                }
            }
        }
        output
    }

    pub fn symbols_with_data_type_name(&self, data_type_name: &str) -> Vec<String> {
        let mut output = Vec::new();
        let filter = Filter {
            filter: &|symbol| symbol.data_type_name == data_type_name,
        };
        for symbol in self.symbols.0.iter() {
            if symbol.1.data_type_name == data_type_name {
                output.push(symbol.0.to_string());
            } else {
                let data_type = if let Some(dt) = self.data_types.0.get(&symbol.1.data_type_name) {
                    dt
                } else {
                    continue;
                };
                let symbols = self.recurse(&filter, data_type);
                for s0 in symbols {
                    output.extend(s0.to_strings(symbol.0.to_string()));
                }
            }
        }
        output
    }

    fn recurse(&self, filter: &Filter, data_type: &DataType) -> Vec<Field> {
        let mut output = Vec::new();
        if let Some(array_range) = data_type.array_ranges.first() {
            let parent_name =
                if let Ok(parent_name) = super::data_type_get_base_name(&data_type.name, Some(1)) {
                    parent_name
                } else {
                    return output;
                };
            let parent_data_type = if let Ok(parent_data_type) = self.data_types().get(parent_name)
            {
                parent_data_type
            } else {
                return output;
            };
            output.extend(
                self.recurse(filter, parent_data_type)
                    .iter()
                    .map(|p| Field::Array(array_range.clone(), Box::new(p.clone()))),
            );
        } else {
            for field in &data_type.fields {
                if (filter.filter)(field) {
                    output.push(Field::End(field.name.to_string()));
                } else if let Ok(child_data_type) = self.data_types.get(&field.data_type_name) {
                    output.extend(
                        self.recurse(filter, child_data_type)
                            .iter()
                            .map(|p| Field::Flat(field.name.to_string(), Box::new(p.clone()))),
                    );
                }
            }
        }
        output
    }
}

struct Filter<'a> {
    filter: &'a dyn Fn(&Symbol) -> bool,
}

#[derive(Clone, Debug)]
enum Field {
    // TcUnit has some very big arrays!
    // To cope with this, just look at the first member of each array, and propogate the results.
    Array(RangeInclusive<i32>, Box<Field>),
    Flat(String, Box<Field>),
    End(String),
}

impl Field {
    fn to_strings(&self, start: String) -> Vec<String> {
        let mut output = vec![start];
        self.to_string_inner(&mut output);
        output
    }

    fn to_string_inner(&self, s: &mut Vec<String>) {
        match self {
            Self::Array(range, next) => {
                let mut s_new = Vec::new();
                for i in *range.start()..=*range.end() {
                    s_new.extend(
                        s.iter()
                            .map(|start| format!("{start}[{i}]"))
                            .collect::<Vec<String>>(),
                    );
                }
                *s = s_new;
                next.to_string_inner(s);
            }
            Self::Flat(middle, next) => {
                s.iter_mut()
                    .for_each(|start| *start = format!("{start}.{middle}"));
                next.to_string_inner(s);
            }
            Self::End(end) => {
                s.iter_mut()
                    .for_each(|start| *start = format!("{start}.{end}"));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flat() {
        let field = Field::Flat(
            String::from("something"),
            Box::new(Field::End(String::from("another_thing"))),
        );
        assert_eq!(
            field.to_strings(String::from("first_thing")),
            vec![String::from("first_thing.something.another_thing")]
        );
    }

    #[test]
    fn array_start() {
        let field = Field::Array(
            RangeInclusive::new(0, 1),
            Box::new(Field::End(String::from("end"))),
        );
        assert_eq!(
            field.to_strings(String::from("start")),
            vec![String::from("start[0].end"), String::from("start[1].end")]
        );
    }

    #[test]
    fn arrays() {
        let field = Field::Flat(
            String::from("many"),
            Box::new(Field::Array(
                RangeInclusive::new(-1, 0),
                Box::new(Field::Array(
                    RangeInclusive::new(0, 2),
                    Box::new(Field::Flat(
                        String::from("items"),
                        Box::new(Field::Array(
                            RangeInclusive::new(-8, -7),
                            Box::new(Field::End(String::from("together"))),
                        )),
                    )),
                )),
            )),
        );
        assert_eq!(
            field.to_strings(String::from("very")),
            vec![
                String::from("very.many[-1][0].items[-8].together"),
                String::from("very.many[0][0].items[-8].together"),
                String::from("very.many[-1][1].items[-8].together"),
                String::from("very.many[0][1].items[-8].together"),
                String::from("very.many[-1][2].items[-8].together"),
                String::from("very.many[0][2].items[-8].together"),
                String::from("very.many[-1][0].items[-7].together"),
                String::from("very.many[0][0].items[-7].together"),
                String::from("very.many[-1][1].items[-7].together"),
                String::from("very.many[0][1].items[-7].together"),
                String::from("very.many[-1][2].items[-7].together"),
                String::from("very.many[0][2].items[-7].together"),
            ]
        )
    }
}
