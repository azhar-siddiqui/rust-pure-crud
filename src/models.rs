use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub description: String,
}

pub struct CrudApp {
    pub items: Vec<Item>,
    pub next_id: u32,
    pub input_name: String,
    pub input_description: String,
    pub editing_id: Option<u32>,
    pub search_query: String,
    pub pending_delete: Option<u32>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl Default for CrudApp {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
            input_name: String::new(),
            input_description: String::new(),
            editing_id: None,
            search_query: String::new(),
            pending_delete: None,
            message: None,
            error: None,
        }
    }
}

impl CrudApp {
    pub fn save_item(&mut self) {
        let name = self.input_name.trim().to_string();
        let description = self.input_description.trim().to_string();

        if name.is_empty() {
            self.error = Some("Please enter a resource name.".into());
            return;
        }

        if name.len() > 100 {
            self.error = Some("Name must be 100 characters or fewer.".into());
            return;
        }

        if description.len() > 1000 {
            self.error = Some("Description must be 1000 characters or fewer.".into());
            return;
        }

        if let Some(id) = self.editing_id {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.name = name;
                item.description = description;
            }

            self.message = Some("Resource updated successfully.".into());
        } else {
            let id = self.next_id;

            self.items.push(Item {
                id,
                name,
                description,
            });

            self.next_id = self.next_id.saturating_add(1);
            self.message = Some("Resource created successfully.".into());
        }

        self.cancel_edit();
        self.error = None;
    }

    pub fn start_edit(&mut self, id: u32) {
        if let Some(item) = self.items.iter().find(|i| i.id == id) {
            self.input_name = item.name.clone();
            self.input_description = item.description.clone();
            self.editing_id = Some(id);
            self.error = None;
            self.message = None;
        }
    }

    pub fn cancel_edit(&mut self) {
        self.input_name.clear();
        self.input_description.clear();
        self.editing_id = None;
        self.error = None;
    }

    pub fn delete_item(&mut self, id: u32) {
        self.items.retain(|item| item.id != id);

        if self.editing_id == Some(id) {
            self.cancel_edit();
        }

        self.pending_delete = None;
        self.message = Some("Resource deleted successfully.".into());
        self.error = None;
    }

    pub fn filtered_items(&self) -> Vec<Item> {
        let query = self.search_query.trim().to_lowercase();

        self.items
            .iter()
            .filter(|item| {
                query.is_empty()
                    || item.name.to_lowercase().contains(&query)
                    || item.description.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    pub fn load_items(&mut self, items: Vec<Item>) {
        self.next_id = items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);

        self.items = items;
    }
}
