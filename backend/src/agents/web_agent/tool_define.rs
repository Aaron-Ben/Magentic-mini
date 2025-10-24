use crate::tools::tool_metadata::{load_tool, make_approval_prompt, ToolSchema};

// --- Approval Prompt (used elsewhere) ---
pub const IRREVERSIBLE_ACTION_PROMPT: &str = 
    include_str!("irreversible_prompt.txt");

// But better: compute at init time or use const fn (not possible with format!)
// So we provide a function:
pub fn irreversible_action_prompt() -> String {
    make_approval_prompt(
        &["buying a product", "submitting a form"],
        &["navigating a website", "things that can be undone"],
        Some("irreversible actions"),
    )
}

// --- Tool JSON Definitions ---
const TOOL_VISIT_URL_JSON: &str = r#"{
    "function": {
        "name": "visit_url",
        "description": "Navigate directly to a provided URL using the browser's address bar. Prefer this tool over other navigation techniques in cases where the user provides a fully-qualified URL (e.g., choose it over clicking links, or inputing queries into search boxes).",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "url": { "type": "string", "description": "The URL to visit in the browser." }
            },
            "required": ["explanation", "url"]
        }
    },
    "metadata": { "requires_approval": "maybe" }
}"#;

const TOOL_WEB_SEARCH_JSON: &str = r#"{
    "function": {
        "name": "web_search",
        "description": "Performs a web search on Bing.com with the given query. Make sure the query is simple and don't use compound queries.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "query": { "type": "string", "description": "The web search query to use." }
            },
            "required": ["explanation", "query"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_HISTORY_BACK_JSON: &str = r#"{
    "function": {
        "name": "history_back",
        "description": "Navigates back one page in the browser's history. This is equivalent to clicking the browser back button.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "maybe" }
}"#;

const TOOL_REFRESH_PAGE_JSON: &str = r#"{
    "function": {
        "name": "refresh_page",
        "description": "Refreshes the current page in the browser. This is equivalent to clicking the browser refresh button or pressing F5.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_PAGE_UP_JSON: &str = r#"{
    "function": {
        "name": "page_up",
        "description": "Scrolls the entire browser viewport one page UP towards the beginning.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_PAGE_DOWN_JSON: &str = r#"{
    "function": {
        "name": "page_down",
        "description": "Scrolls the entire browser viewport one page DOWN towards the end.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_SCROLL_DOWN_JSON: &str = r#"{
    "function": {
        "name": "scroll_down",
        "description": "Scrolls down on the current page using mouse wheel for 400 pixels.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_SCROLL_UP_JSON: &str = r#"{
    "function": {
        "name": "scroll_up",
        "description": "Scrolls up on the current page using mouse wheel for 400 pixels.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_CLICK_JSON: &str = r#"{
    "function": {
        "name": "click",
        "description": "Clicks the mouse on the target with the given id.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "target_id": { "type": "integer", "description": "The numeric id of the target to click." }
            },
            "required": ["explanation", "target_id"]
        }
    },
    "metadata": { "requires_approval": "maybe" }
}"#;

const TOOL_CLICK_FULL_JSON: &str = r#"{
    "function": {
        "name": "click_full",
        "description": "Clicks the mouse on the target with the given id, with optional hold duration and button type.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "target_id": { "type": "integer", "description": "The numeric id of the target to click." },
                "hold": { "type": "number", "description": "Seconds to hold the mouse button down before releasing. Default: 0.0.", "default": 0.0 },
                "button": { "type": "string", "enum": ["left", "right"], "description": "Mouse button to use. Default: 'left'.", "default": "left" }
            },
            "required": ["explanation", "target_id", "hold", "button"]
        }
    },
    "metadata": { "requires_approval": "maybe" }
}"#;

const TOOL_INPUT_TEXT_JSON: &str = r#"{
    "function": {
        "name": "input_text",
        "description": "Types the given text value into the specified field. Presses enter only if you want to submit the form or search.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "input_field_id": { "type": "integer", "description": "The numeric id of the input field to receive the text." },
                "text_value": { "type": "string", "description": "The text to type into the input field." },
                "press_enter": { "type": "boolean", "description": "Whether to press enter after typing into the field or not." },
                "delete_existing_text": { "type": "boolean", "description": "Whether to delete existing text in the field before inputing the text value." }
            },
            "required": ["explanation", "input_field_id", "text_value", "delete_existing_text"]
        }
    },
    "metadata": { "requires_approval": "maybe" }
}"#;

const TOOL_SCROLL_ELEMENT_DOWN_JSON: &str = r#"{
    "function": {
        "name": "scroll_element_down",
        "description": "Scrolls a given html element (e.g., a div or a menu) DOWN.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "target_id": { "type": "integer", "description": "The numeric id of the target to scroll down." }
            },
            "required": ["explanation", "target_id"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_SCROLL_ELEMENT_UP_JSON: &str = r#"{
    "function": {
        "name": "scroll_element_up",
        "description": "Scrolls a given html element (e.g., a div or a menu) UP.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "target_id": { "type": "integer", "description": "The numeric id of the target to scroll up." }
            },
            "required": ["explanation", "target_id"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_HOVER_JSON: &str = r#"{
    "function": {
        "name": "hover",
        "description": "Hovers the mouse over the target with the given id.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "target_id": { "type": "integer", "description": "The numeric id of the target to hover over." }
            },
            "required": ["explanation", "target_id"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;


const TOOL_ANSWER_QUESTION_JSON: &str = r#"{
    "function": {
        "name": "answer_question",
        "description": "Answers a question about the current webpage. Use this tool when the user asks a question about the current webpage.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "question": { "type": "string", "description": "The question to answer." }
            },
            "required": ["explanation", "question"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

const TOOL_SUMMARIZE_PAGE_JSON: &str = r#"{
    "function": {
        "name": "summarize_page",
        "description": "Summarizes the current webpage. Use this tool when the user asks to summarize the current webpage.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

const TOOL_SLEEP_JSON: &str = r#"{
    "function": {
        "name": "sleep",
        "description": "Sleeps for a given number of seconds. Use this tool when the user asks to sleep for a given number of seconds.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "seconds": { "type": "integer", "description": "The number of seconds to sleep." }
            },
            "required": ["explanation", "seconds"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

const TOOL_STOP_ACTION_JSON: &str = r#"{
    "function": {
        "name": "stop_action",
        "description": "Stops the current action. Use this tool when the user asks to stop the current action.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

const TOOL_SELECT_OPTION_JSON: &str = r#"{
    "function": {
        "name": "select_option",
        "description": "Selects an option from a dropdown menu.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "target_id": { "type": "integer", "description": "The numeric id of the target to select an option from." }
            },
            "required": ["explanation", "target_id"]
        }
    },
    "metadata": { "requires_approval": "never" }
}"#;

const TOOL_CREATE_TAB_JSON: &str = r#"{
    "function": {
        "name": "create_tab",
        "description": "Creates a new browser tab.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." }
            },
            "required": ["explanation"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

const TOOL_SWITCH_TAB_JSON: &str = r#"{
    "function": {
        "name": "switch_tab",
        "description": "Switches to the specified browser tab by its index.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "tab_index": { "type": "integer", "description": "The index of the tab to switch to (0-based)." }
            },
            "required": ["explanation", "tab_index"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

const TOOL_CLOSE_TAB_JSON: &str = r#"{
    "function": {
        "name": "close_tab",
        "description": "Closes the specified browser tab by its index and switches to an adjacent tab. Cannot close the last remaining tab.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "Explain to the user the action to be performed and reason for doing so. Phrase as if you are directly talking to the user." },
                "tab_index": { "type": "integer", "description": "The index of the tab to close (0-based)." }
            },
            "required": ["explanation", "tab_index"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

const TOOL_UPLOAD_FILE_JSON: &str = r#"{
    "function": {
        "name": "upload_file",
        "description": "Upload a file to a specified input element.",
        "parameters": {
            "type": "object",
            "properties": {
                "explanation": { "type": "string", "description": "The explanation of the action to be performed." },
                "target_id": { "type": "string", "description": "The ID of the target input element." },
                "file_path": { "type": "string", "description": "The path to the file to be uploaded." }
            },
            "required": ["explanation", "target_id", "file_path"]
        }
    },
    "metadata": { "requires_approval": "always" }
}"#;

// --- Public Tool Instances (lazy init or init-once) ---
// Since Rust doesn't have module-level mutable state easily,
// we provide a function to initialize all tools.

pub struct DefaultTools {
    pub visit_url: ToolSchema,
    pub web_search: ToolSchema,
    pub history_back: ToolSchema,
    pub refresh_page: ToolSchema,
    pub page_up: ToolSchema,
    pub page_down: ToolSchema,
    pub scroll_down: ToolSchema,
    pub scroll_up: ToolSchema,
    pub click: ToolSchema,
    pub click_full: ToolSchema,
    pub input_text: ToolSchema, // note: name is "input_text" in JSON
    pub scroll_element_down: ToolSchema,
    pub scroll_element_up: ToolSchema,
    pub hover: ToolSchema,
    pub answer_question: ToolSchema, // name: "answer_question"
    pub summarize_page: ToolSchema,
    pub sleep: ToolSchema,
    pub stop_action: ToolSchema,
    pub select_option: ToolSchema,
    pub create_tab: ToolSchema,
    pub switch_tab: ToolSchema,
    pub close_tab: ToolSchema,
    pub upload_file: ToolSchema,
}

impl DefaultTools {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            visit_url: load_tool(TOOL_VISIT_URL_JSON)?,
            web_search: load_tool(TOOL_WEB_SEARCH_JSON)?,
            history_back: load_tool(TOOL_HISTORY_BACK_JSON)?,
            refresh_page: load_tool(TOOL_REFRESH_PAGE_JSON)?,
            page_up: load_tool(TOOL_PAGE_UP_JSON)?,
            page_down: load_tool(TOOL_PAGE_DOWN_JSON)?,
            scroll_down: load_tool(TOOL_SCROLL_DOWN_JSON)?,
            scroll_up: load_tool(TOOL_SCROLL_UP_JSON)?,
            click: load_tool(TOOL_CLICK_JSON)?,
            click_full: load_tool(TOOL_CLICK_FULL_JSON)?,
            input_text: load_tool(TOOL_INPUT_TEXT_JSON)?,
            scroll_element_down: load_tool(TOOL_SCROLL_ELEMENT_DOWN_JSON)?,
            scroll_element_up: load_tool(TOOL_SCROLL_ELEMENT_UP_JSON)?,
            hover: load_tool(TOOL_HOVER_JSON)?,
            answer_question: load_tool(TOOL_ANSWER_QUESTION_JSON)?,
            summarize_page: load_tool(TOOL_SUMMARIZE_PAGE_JSON)?,
            sleep: load_tool(TOOL_SLEEP_JSON)?,
            stop_action: load_tool(TOOL_STOP_ACTION_JSON)?,
            select_option: load_tool(TOOL_SELECT_OPTION_JSON)?,
            create_tab: load_tool(TOOL_CREATE_TAB_JSON)?,
            switch_tab: load_tool(TOOL_SWITCH_TAB_JSON)?,
            close_tab: load_tool(TOOL_CLOSE_TAB_JSON)?,
            upload_file: load_tool(TOOL_UPLOAD_FILE_JSON)?,
        })
    }
}