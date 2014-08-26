use uuid::Uuid;
use serialize::json;

#[deriving(Show, Clone, Encodable)]
pub struct Todo {
    pub title: String,
    pub order: Option<f64>,
    pub completed: bool,
    pub id: Uuid,
    pub url: String // url::Url is not encodable
}

impl Todo {
    pub fn new_from_json_str(unparsed_json: &str) -> Result<Todo, String> {
        match valid_fresh_todo_json(unparsed_json) {
            Ok(json) => {
                let id = Uuid::new_v4();
                Ok(Todo {
                    title: json.find(&"title".to_string()).unwrap().as_string().unwrap().to_string(),
                    order: json.find(&"order".to_string()).and_then(|j| j.as_f64()),
                    completed: false,
                    id: id,
                    url: format!("http://localhost:3000/{}", id)
                })
            },
            Err(msg) => Err(msg)
        }
    }

    pub fn update_from_json_str(&mut self, unparsed_json: &str) -> Result<(), String> {
        match json::from_str(unparsed_json) {
            Ok(json) => {
                match json.find(&"title".to_string()) {
                    Some(title) => self.title = title.as_string().unwrap().to_string(),
                    None => {}
                }
                match json.find(&"completed".to_string()) {
                    Some(c) => self.completed = c.as_boolean().unwrap(),
                    None => {}
                }
                match json.find(&"order".to_string()) {
                    Some(o) => self.order = o.as_f64(),
                    None => {}
                }
                Ok(())
            }
            Err(builder_error) => {
                Err(format!("Failed to parse JSON: {}", builder_error))
            }
        }

    }
}

fn valid_fresh_todo_json(unparsed_json: &str) -> Result<json::Json, String> {
    match json::from_str(unparsed_json) {
        Ok(json) => {
            match json.find(&"title".to_string()) {
                Some(title) => if !title.is_string() {
                    return Err("title must be a string".to_string())
                },
                None => return Err("title is required".to_string())
            }
            match json.find(&"order".to_string()) {
                Some(order) => if order.is_number() {
                    Ok(json.clone())
                } else {
                    Err("order must be a number".to_string())
                },
                None => Ok(json.clone())
            }
        },
        Err(builder_error) => Err(format!("Failed to parse JSON: {}", builder_error))
    }
}

#[cfg(test)]
mod Todo_new_from_json {
    use super::Todo;

    #[test]
    fn parses_todo_with_order() {
        let todo = Todo::new_from_json_str("{\"title\": \"a todo\", \"order\":100}").unwrap();
        assert_eq!("a todo".to_string(), todo.title);
        assert_eq!(100f64, todo.order.unwrap());
    }

    #[test]
    fn parses_todo_with_only_title() {
        let todo = Todo::new_from_json_str("{\"title\": \"a todo\"}").unwrap();
        assert_eq!("a todo".to_string(), todo.title);
        assert_eq!(None, todo.order);
    }

    #[test]
    fn errs_with_details_on_missing_title() {
        assert_eq!("title is required".to_string(), Todo::new_from_json_str("{}").err().unwrap());
    }

    #[test]
    fn errs_with_details_on_malformed_json() {
        assert_eq!("Failed to parse JSON: SyntaxError(EOF While parsing value, 1, 10)".to_string(),
                   Todo::new_from_json_str("{\"title\":").err().unwrap());
    }
}
