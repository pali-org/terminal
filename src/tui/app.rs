//! TUI application state and logic

use anyhow::Result;
use pali_types::Todo;
use ratatui::widgets::ListState;
use crate::{ApiClient, Config};
use crate::tui::components::InputForm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    TodoList,
    AddTodo,
    EditTodo,
    Help,
    Settings,
    Search,
    TodoDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub should_quit: bool,
    pub api_client: ApiClient,
    pub config: Config,
    pub current_screen: AppScreen,
    pub input_mode: InputMode,
    pub todos: Vec<Todo>,
    pub selected_todo: Option<usize>,
    pub list_state: ListState,  // Moved from UI to app state for performance
    pub input_buffer: String,
    pub input_form: InputForm,  // Advanced form for add/edit
    pub loading: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    // Search and filtering state
    pub search_query: String,
    pub show_all_todos: bool,
    pub filter_priority: Option<i32>,
    pub filter_tag: Option<String>,
    pub filtered_todos: Vec<Todo>,  // Cache filtered results
}

impl App {
    /// Creates a new TUI application instance with loaded configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Configuration cannot be loaded from disk
    /// - Configuration file format is invalid
    /// - API client initialization fails
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let api_client = ApiClient::new()?;
        
        let mut app = Self {
            should_quit: false,
            api_client,
            config,
            current_screen: AppScreen::TodoList,
            input_mode: InputMode::Normal,
            todos: Vec::new(),
            selected_todo: None,
            list_state: ListState::default(),
            input_buffer: String::new(),
            input_form: InputForm::new(),
            loading: false,
            error_message: None,
            success_message: None,
            // Initialize search and filtering
            search_query: String::new(),
            show_all_todos: false,
            filter_priority: None,
            filter_tag: None,
            filtered_todos: Vec::new(),
        };
        
        // Apply initial filters
        app.apply_filters();
        
        Ok(app)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.success_message = None;
    }

    pub fn show_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.error_message = None;
    }

    /// Applies current search query and filters to update filtered_todos
    pub fn apply_filters(&mut self) {
        self.filtered_todos = self.todos
            .iter()
            .filter(|todo| {
                // Apply completion filter
                if !self.show_all_todos && todo.completed {
                    return false;
                }
                
                // Apply search query filter
                if !self.search_query.is_empty() {
                    let query_lower = self.search_query.to_lowercase();
                    let title_match = todo.title.to_lowercase().contains(&query_lower);
                    let desc_match = todo.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false);
                    
                    if !title_match && !desc_match {
                        return false;
                    }
                }
                
                // Apply priority filter
                if let Some(priority) = self.filter_priority {
                    if todo.priority != priority {
                        return false;
                    }
                }
                
                // Apply tag filter (placeholder - tags not fully implemented yet)
                // if let Some(tag) = &self.filter_tag {
                //     // TODO: Implement tag filtering when tags are added
                // }
                
                true
            })
            .cloned()
            .collect();
            
        // Reset selection when filters change
        if self.filtered_todos.is_empty() {
            self.selected_todo = None;
            self.list_state.select(None);
        } else {
            self.selected_todo = Some(0);
            self.list_state.select(Some(0));
        }
    }

    /// Starts search mode
    pub fn start_search(&mut self) {
        self.current_screen = AppScreen::Search;
        self.input_mode = InputMode::Editing;
        self.search_query.clear();
        self.clear_messages();
    }

    /// Executes search with current query
    pub async fn execute_search(&mut self) -> Result<()> {
        if self.search_query.trim().is_empty() {
            // Empty search - show all todos
            self.current_screen = AppScreen::TodoList;
            self.input_mode = InputMode::Normal;
            self.apply_filters();
            return Ok(());
        }

        self.loading = true;
        self.clear_messages();
        
        match self.api_client.search_todos(&self.search_query).await {
            Ok(todos) => {
                self.todos = todos;
                self.apply_filters();
                self.current_screen = AppScreen::TodoList;
                self.input_mode = InputMode::Normal;
                self.show_success(format!("Found {} results for '{}'", self.filtered_todos.len(), self.search_query));
            }
            Err(_) => {
                self.show_error("Search failed. Please try again.".to_string());
            }
        }
        
        self.loading = false;
        Ok(())
    }

    /// Toggles between showing all todos and only pending todos
    pub fn toggle_show_all(&mut self) {
        self.show_all_todos = !self.show_all_todos;
        self.apply_filters();
        let status = if self.show_all_todos { "all todos" } else { "pending todos" };
        self.show_success(format!("Now showing {}", status));
    }

    /// Sets priority filter (None to clear filter)
    pub fn set_priority_filter(&mut self, priority: Option<i32>) {
        self.filter_priority = priority;
        self.apply_filters();
        let msg = match priority {
            Some(1) => "Filtering by low priority".to_string(),
            Some(2) => "Filtering by medium priority".to_string(),
            Some(3) => "Filtering by high priority".to_string(),
            None => "Priority filter cleared".to_string(),
            _ => "Invalid priority".to_string(),
        };
        self.show_success(msg);
    }

    /// Shows detailed view of currently selected todo
    pub fn show_todo_detail(&mut self) {
        if self.selected_todo.is_some() {
            self.current_screen = AppScreen::TodoDetail;
        }
    }

    pub fn next_todo(&mut self) {
        if !self.filtered_todos.is_empty() {
            let i = match self.selected_todo {
                Some(i) => {
                    if i >= self.filtered_todos.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.selected_todo = Some(i);
            self.list_state.select(Some(i));
        }
    }

    pub fn previous_todo(&mut self) {
        if !self.filtered_todos.is_empty() {
            let i = match self.selected_todo {
                Some(i) => {
                    if i == 0 {
                        self.filtered_todos.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_todo = Some(i);
            self.list_state.select(Some(i));
        }
    }

    /// Loads todos from the API server
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails (displays error message to user)
    /// - API returns an error response (displays error message to user)
    /// - Response parsing fails (displays error message to user)
    /// 
    /// Note: Errors are shown to the user via UI messages and don't propagate
    pub async fn load_todos(&mut self) -> Result<()> {
        self.loading = true;
        self.clear_messages();
        
        match self.api_client.list_todos(None, None).await {
            Ok(todos) => {
                self.todos = todos;
                self.apply_filters();  // Apply current filters
                // Safe bounds checking without unwrap and sync list_state
                if let Some(selected_index) = self.selected_todo {
                    if selected_index >= self.filtered_todos.len() {
                        let new_selection = if self.filtered_todos.is_empty() { None } else { Some(0) };
                        self.selected_todo = new_selection;
                        self.list_state.select(new_selection);
                    }
                } else if !self.filtered_todos.is_empty() {
                    // Auto-select first item if none selected but todos exist
                    self.selected_todo = Some(0);
                    self.list_state.select(Some(0));
                }
                self.show_success(format!("Loaded {} todo(s), showing {}", self.todos.len(), self.filtered_todos.len()));
            }
            Err(_) => {
                self.show_error("Unable to load todos. Please check your connection and try again.".to_string());
            }
        }
        
        self.loading = false;
        Ok(())
    }

    /// Toggles the completion status of the currently selected todo
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails (displays error message to user)
    /// - API returns an error response (displays error message to user)
    /// - Selected todo no longer exists on server (displays error message to user)
    /// 
    /// Note: Errors are shown to the user via UI messages and don't propagate
    pub async fn toggle_selected_todo(&mut self) -> Result<()> {
        if let Some(index) = self.selected_todo {
            if let Some(todo) = self.filtered_todos.get(index) {
                let todo_id = todo.id.clone();
                self.loading = true;
                self.clear_messages();
                
                match self.api_client.toggle_todo(&todo_id).await {
                    Ok(updated_todo) => {
                        // Update in main todos list
                        if let Some(main_index) = self.todos.iter().position(|t| t.id == todo_id) {
                            self.todos[main_index] = updated_todo.clone();
                        }
                        // Update in filtered list
                        self.filtered_todos[index] = updated_todo;
                        self.show_success("Todo toggled successfully".to_string());
                    }
                    Err(_) => {
                        self.show_error("Unable to update todo status. Please try again.".to_string());
                    }
                }
                
                self.loading = false;
            }
        }
        Ok(())
    }

    /// Deletes the currently selected todo from the server
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails (displays error message to user)
    /// - API returns an error response (displays error message to user)
    /// - Selected todo no longer exists on server (displays error message to user)
    /// 
    /// Note: Errors are shown to the user via UI messages and don't propagate
    pub async fn delete_selected_todo(&mut self) -> Result<()> {
        if let Some(index) = self.selected_todo {
            if let Some(todo) = self.filtered_todos.get(index) {
                let todo_id = todo.id.clone();
                let todo_title = todo.title.clone();
                self.loading = true;
                self.clear_messages();
                
                match self.api_client.delete_todo(&todo_id).await {
                    Ok(()) => {
                        // Remove from main todos list
                        self.todos.retain(|t| t.id != todo_id);
                        // Remove from filtered list
                        self.filtered_todos.remove(index);
                        
                        // Update selection
                        if self.filtered_todos.is_empty() {
                            self.selected_todo = None;
                            self.list_state.select(None);
                        } else if index >= self.filtered_todos.len() {
                            let new_index = self.filtered_todos.len() - 1;
                            self.selected_todo = Some(new_index);
                            self.list_state.select(Some(new_index));
                        }
                        self.show_success(format!("Deleted: {todo_title}"));
                    }
                    Err(_) => {
                        self.show_error("Unable to delete todo. Please try again.".to_string());
                    }
                }
                
                self.loading = false;
            }
        }
        Ok(())
    }

    /// Starts editing the currently selected todo
    /// 
    /// # Errors
    /// 
    /// Returns an error if no todo is selected
    pub async fn start_edit_selected_todo(&mut self) -> Result<()> {
        if let Some(index) = self.selected_todo {
            if let Some(todo) = self.filtered_todos.get(index) {
                // Pre-populate the form with current todo data
                self.input_form.title = todo.title.clone();
                self.input_form.description = todo.description.clone().unwrap_or_default();
                self.input_form.priority = todo.priority;
                
                // Pre-populate due date if present
                self.input_form.due_date = if let Some(due_ts) = todo.due_date {
                    chrono::DateTime::from_timestamp(due_ts, 0)
                        .map(|dt| dt.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                
                self.current_screen = AppScreen::EditTodo;
                self.input_mode = InputMode::Editing;
                self.clear_messages();
            }
        }
        Ok(())
    }

    /// Updates the currently selected todo with form data
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails (displays error message to user)
    /// - API returns an error response (displays error message to user)
    /// - Selected todo no longer exists on server (displays error message to user)
    /// 
    /// Note: Errors are shown to the user via UI messages and don't propagate
    pub async fn update_selected_todo(&mut self) -> Result<()> {
        if !self.input_form.is_valid() {
            self.show_error("Please enter a title for your todo".to_string());
            return Ok(());
        }

        if let Some(index) = self.selected_todo {
            if let Some(todo) = self.filtered_todos.get(index) {
                let todo_id = todo.id.clone();
                self.loading = true;
                self.clear_messages();
                
                // Parse and validate due date
                let due_date = match self.input_form.parse_due_date() {
                    Ok(due) => due,
                    Err(err) => {
                        self.loading = false;
                        self.show_error(err);
                        return Ok(());
                    }
                };
                
                let update_request = pali_types::UpdateTodoRequest {
                    title: Some(self.input_form.title.trim().to_string()),
                    description: if self.input_form.description.trim().is_empty() { 
                        None 
                    } else { 
                        Some(self.input_form.description.trim().to_string()) 
                    },
                    completed: None,
                    priority: Some(self.input_form.priority),
                    due_date,
                };

                match self.api_client.update_todo(&todo_id, update_request).await {
                    Ok(updated_todo) => {
                        // Update in main todos list
                        if let Some(main_index) = self.todos.iter().position(|t| t.id == todo_id) {
                            self.todos[main_index] = updated_todo.clone();
                        }
                        // Update in filtered list
                        self.filtered_todos[index] = updated_todo.clone();
                        self.input_form.clear();
                        self.current_screen = AppScreen::TodoList;
                        self.input_mode = InputMode::Normal;
                        self.show_success(format!("Updated: {}", updated_todo.title));
                    }
                    Err(_) => {
                        self.show_error("Unable to update todo. Please try again.".to_string());
                    }
                }
                
                self.loading = false;
            }
        }
        Ok(())
    }

    /// Creates a new todo using the input form content
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Network request fails (displays error message to user)
    /// - API returns an error response (displays error message to user)
    /// - Server rejects the todo creation (displays error message to user)
    /// 
    /// Note: Errors are shown to the user via UI messages and don't propagate
    pub async fn create_todo(&mut self) -> Result<()> {
        if !self.input_form.is_valid() {
            self.show_error("Please enter a title for your todo".to_string());
            return Ok(());
        }
        
        self.loading = true;
        self.clear_messages();
        
        let request = match self.input_form.to_create_request() {
            Ok(req) => req,
            Err(err) => {
                self.loading = false;
                self.show_error(err);
                return Ok(());
            }
        };
        
        match self.api_client.create_todo(request).await {
            Ok(todo) => {
                self.todos.push(todo.clone());
                self.apply_filters();  // Reapply filters to include new todo
                // Select the new todo in filtered list if it appears
                if let Some(new_index) = self.filtered_todos.iter().position(|t| t.id == todo.id) {
                    self.selected_todo = Some(new_index);
                    self.list_state.select(Some(new_index));
                }
                self.input_form.clear();
                self.current_screen = AppScreen::TodoList;
                self.input_mode = InputMode::Normal;
                self.show_success(format!("Created: {}", todo.title));
            }
            Err(_) => {
                self.show_error("Unable to create todo. Please try again.".to_string());
            }
        }
        
        self.loading = false;
        Ok(())
    }

    /// Handles keyboard input events
    /// 
    /// # Errors
    /// 
    /// Returns an error if key handling fails
    pub async fn handle_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        self.clear_messages();
        
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Editing => self.handle_editing_key(key).await,
        }
    }

    async fn handle_normal_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        use crossterm::event::KeyCode;
        
        match self.current_screen {
            AppScreen::TodoList => {
                match key {
                    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                    KeyCode::Char('r') => { self.load_todos().await?; },
                    KeyCode::Char('n' | 'a') => {
                        self.current_screen = AppScreen::AddTodo;
                        self.input_mode = InputMode::Editing;
                        self.input_form.clear();
                    },
                    KeyCode::Char('e') => {
                        self.start_edit_selected_todo().await?;
                    },
                    KeyCode::Char('h' | '?') => {
                        self.current_screen = AppScreen::Help;
                    },
                    KeyCode::Char('s') => {
                        self.current_screen = AppScreen::Settings;
                    },
                    KeyCode::Char('/') => {
                        self.start_search();
                    },
                    KeyCode::Char('f') => {
                        self.toggle_show_all();
                    },
                    KeyCode::Char('1') => {
                        self.set_priority_filter(Some(1));
                    },
                    KeyCode::Char('2') => {
                        self.set_priority_filter(Some(2));
                    },
                    KeyCode::Char('3') => {
                        self.set_priority_filter(Some(3));
                    },
                    KeyCode::Char('0') => {
                        self.set_priority_filter(None);
                    },
                    KeyCode::Char('v') => {
                        self.show_todo_detail();
                    },
                    KeyCode::Up | KeyCode::Char('k') => self.previous_todo(),
                    KeyCode::Down | KeyCode::Char('j') => self.next_todo(),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.toggle_selected_todo().await?;
                    },
                    KeyCode::Char('d') => {
                        self.delete_selected_todo().await?;
                    },
                    _ => {}
                }
            },
            AppScreen::Help | AppScreen::Settings | AppScreen::TodoDetail => {
                match key {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.current_screen = AppScreen::TodoList;
                    },
                    _ => {}
                }
            },
            AppScreen::AddTodo | AppScreen::EditTodo | AppScreen::Search => {
                if key == KeyCode::Esc {
                    self.current_screen = AppScreen::TodoList;
                    self.input_mode = InputMode::Normal;
                    self.input_form.clear();
                    self.search_query.clear();
                }
            }
        }
        
        Ok(())
    }

    async fn handle_editing_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        use crossterm::event::KeyCode;
        
        match key {
            KeyCode::Esc => {
                self.current_screen = AppScreen::TodoList;
                self.input_mode = InputMode::Normal;
                self.input_form.clear();
            },
            KeyCode::Enter => {
                match self.current_screen {
                    AppScreen::AddTodo => {
                        self.create_todo().await?;
                    },
                    AppScreen::EditTodo => {
                        self.update_selected_todo().await?;
                    },
                    AppScreen::Search => {
                        self.execute_search().await?;
                    },
                    _ => {}
                }
            },
            KeyCode::Tab => {
                self.input_form.next_field();
            },
            KeyCode::BackTab => {
                self.input_form.previous_field();
            },
            KeyCode::Char(c) => {
                if self.current_screen == AppScreen::Search {
                    self.search_query.push(c);
                } else {
                    self.input_form.handle_char(c);
                }
            },
            KeyCode::Backspace => {
                if self.current_screen == AppScreen::Search {
                    self.search_query.pop();
                } else {
                    self.input_form.handle_backspace();
                }
            },
            _ => {}
        }
        
        Ok(())
    }
}

// Note: Default implementation removed - use App::new() instead
// as config loading can fail and should be handled explicitly