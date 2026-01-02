use common::llm::{Message, Role};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RenderedContext {
    pub messages: Vec<Message>,
    pub estimated_prompt_tokens: u32,
    pub dropped_turns: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContextLimits {
    pub max_prompt_tokens: u32,
    pub reserved_completion_tokens: Option<u32>,
}

pub trait TokenEstimator: Send + Sync {
    fn estimate_message_tokens(&self, message: &Message) -> u32;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultTokenEstimator;

impl TokenEstimator for DefaultTokenEstimator {
    fn estimate_message_tokens(&self, message: &Message) -> u32 {
        let mut chars: usize = 0;

        if let Some(content) = &message.content {
            chars = chars.saturating_add(content.len());
        }

        if let Some(tool_calls) = &message.tool_calls {
            for call in tool_calls {
                chars = chars.saturating_add(call.id.len());
                chars = chars.saturating_add(call.kind.len());
                chars = chars.saturating_add(call.function.name.len());
                chars = chars.saturating_add(call.function.arguments.len());
            }
        }

        if let Some(tool_call_id) = &message.tool_call_id {
            chars = chars.saturating_add(tool_call_id.len());
        }

        let approx = ((chars as u32).saturating_add(3)) / 4;
        approx.max(1)
    }
}

#[derive(Debug)]
pub enum ContextError {
    NoActiveUserTurn,
    AssistantToolCallsMustUseBeginExchange,
    ToolResultMissingCallId,
    ToolResultWithoutOpenExchange,
    ToolResultUnknownCallId {
        tool_call_id: String,
    },
    ToolResultDuplicateCallId {
        tool_call_id: String,
    },
    PromptBudgetExceeded {
        prompt_budget: u32,
        required_tokens: u32,
    },
    InvalidRole {
        expected: Role,
        got: Role,
    },
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextError::NoActiveUserTurn => write!(f, "no active user turn"),
            ContextError::AssistantToolCallsMustUseBeginExchange => {
                write!(
                    f,
                    "assistant tool calls must be appended via begin_tool_exchange"
                )
            }
            ContextError::ToolResultMissingCallId => {
                write!(f, "tool result message missing tool_call_id")
            }
            ContextError::ToolResultWithoutOpenExchange => {
                write!(f, "tool result appended without an open tool exchange")
            }
            ContextError::ToolResultUnknownCallId { tool_call_id } => {
                write!(
                    f,
                    "tool result tool_call_id not present in tool calls: {tool_call_id}"
                )
            }
            ContextError::ToolResultDuplicateCallId { tool_call_id } => {
                write!(
                    f,
                    "tool result tool_call_id already recorded: {tool_call_id}"
                )
            }
            ContextError::PromptBudgetExceeded {
                prompt_budget,
                required_tokens,
            } => write!(
                f,
                "prompt budget exceeded (budget={prompt_budget}, required={required_tokens})"
            ),
            ContextError::InvalidRole { expected, got } => {
                write!(f, "invalid role (expected={expected:?}, got={got:?})")
            }
        }
    }
}

impl std::error::Error for ContextError {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Context {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Entry {
    Pinned { message: Message },
    UserTurn { user: Message, steps: Vec<Step> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Step {
    Assistant {
        message: Message,
    },
    ToolExchange {
        assistant: Message,
        tool_results: Vec<Message>,
    },
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn push_pinned(&mut self, message: Message) {
        self.entries.push(Entry::Pinned { message });
    }

    pub fn push_user(&mut self, message: Message) -> Result<(), ContextError> {
        if message.role != Role::User {
            return Err(ContextError::InvalidRole {
                expected: Role::User,
                got: message.role,
            });
        }
        self.entries.push(Entry::UserTurn {
            user: message,
            steps: Vec::new(),
        });
        Ok(())
    }

    pub fn push_assistant(&mut self, message: Message) -> Result<(), ContextError> {
        if message.role != Role::Assistant {
            return Err(ContextError::InvalidRole {
                expected: Role::Assistant,
                got: message.role,
            });
        }

        if message
            .tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
        {
            return Err(ContextError::AssistantToolCallsMustUseBeginExchange);
        }

        let steps = self.current_steps_mut()?;
        steps.push(Step::Assistant { message });
        Ok(())
    }

    pub fn begin_tool_exchange(&mut self, assistant: Message) -> Result<(), ContextError> {
        if assistant.role != Role::Assistant {
            return Err(ContextError::InvalidRole {
                expected: Role::Assistant,
                got: assistant.role,
            });
        }
        if !assistant
            .tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
        {
            return Err(ContextError::AssistantToolCallsMustUseBeginExchange);
        }

        let steps = self.current_steps_mut()?;
        steps.push(Step::ToolExchange {
            assistant,
            tool_results: Vec::new(),
        });
        Ok(())
    }

    pub fn push_tool_result(&mut self, tool_result: Message) -> Result<(), ContextError> {
        if tool_result.role != Role::Tool {
            return Err(ContextError::InvalidRole {
                expected: Role::Tool,
                got: tool_result.role,
            });
        }

        let tool_call_id = tool_result
            .tool_call_id
            .clone()
            .ok_or(ContextError::ToolResultMissingCallId)?;

        let exchange = self.current_open_exchange_mut()?;

        let valid_call_ids: Vec<&str> = exchange
            .assistant
            .tool_calls
            .as_ref()
            .map(|calls| calls.iter().map(|c| c.id.as_str()).collect())
            .unwrap_or_default();

        if !valid_call_ids.iter().any(|id| *id == tool_call_id) {
            return Err(ContextError::ToolResultUnknownCallId { tool_call_id });
        }

        if exchange
            .tool_results
            .iter()
            .any(|m| m.tool_call_id.as_deref() == Some(tool_call_id.as_str()))
        {
            return Err(ContextError::ToolResultDuplicateCallId { tool_call_id });
        }

        exchange.tool_results.push(tool_result);
        Ok(())
    }

    pub fn push_message(&mut self, message: Message) -> Result<(), ContextError> {
        match message.role {
            Role::System => {
                self.push_pinned(message);
                Ok(())
            }
            Role::User => self.push_user(message),
            Role::Assistant => {
                if message
                    .tool_calls
                    .as_ref()
                    .is_some_and(|calls| !calls.is_empty())
                {
                    self.begin_tool_exchange(message)
                } else {
                    self.push_assistant(message)
                }
            }
            Role::Tool => self.push_tool_result(message),
        }
    }

    pub fn linear_messages(&self) -> Vec<Message> {
        let mut out = Vec::new();
        for entry in &self.entries {
            match entry {
                Entry::Pinned { message } => out.push(message.clone()),
                Entry::UserTurn { user, steps } => {
                    out.push(user.clone());
                    for step in steps {
                        match step {
                            Step::Assistant { message } => out.push(message.clone()),
                            Step::ToolExchange {
                                assistant,
                                tool_results,
                            } => {
                                out.push(assistant.clone());
                                out.extend(tool_results.iter().cloned());
                            }
                        }
                    }
                }
            }
        }
        out
    }

    pub fn render(
        &self,
        system_messages: &[Message],
        limits: Option<ContextLimits>,
        estimator: &dyn TokenEstimator,
    ) -> Result<RenderedContext, ContextError> {
        let system_tokens: u32 = system_messages
            .iter()
            .map(|m| estimator.estimate_message_tokens(m))
            .sum();

        let mut entry_tokens: Vec<u32> = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            entry_tokens.push(estimate_entry_tokens(entry, estimator));
        }

        let mut total_tokens: u32 =
            system_tokens.saturating_add(entry_tokens.iter().copied().sum::<u32>());

        let mut keep: Vec<bool> = vec![true; self.entries.len()];
        let mut dropped_turns: u32 = 0;

        if let Some(limits) = limits {
            let prompt_budget = limits
                .max_prompt_tokens
                .saturating_sub(limits.reserved_completion_tokens.unwrap_or(0));

            if total_tokens > prompt_budget {
                for (idx, entry) in self.entries.iter().enumerate() {
                    if total_tokens <= prompt_budget {
                        break;
                    }

                    if matches!(entry, Entry::UserTurn { .. }) {
                        keep[idx] = false;
                        total_tokens = total_tokens.saturating_sub(entry_tokens[idx]);
                        dropped_turns = dropped_turns.saturating_add(1);
                    }
                }
            }

            if total_tokens > prompt_budget {
                return Err(ContextError::PromptBudgetExceeded {
                    prompt_budget,
                    required_tokens: total_tokens,
                });
            }
        }

        let mut messages = Vec::new();
        messages.extend(system_messages.iter().cloned());
        for (idx, entry) in self.entries.iter().enumerate() {
            if !keep[idx] {
                continue;
            }
            render_entry(entry, &mut messages);
        }

        Ok(RenderedContext {
            messages,
            estimated_prompt_tokens: total_tokens,
            dropped_turns,
        })
    }

    fn current_steps_mut(&mut self) -> Result<&mut Vec<Step>, ContextError> {
        match self.entries.last_mut() {
            Some(Entry::UserTurn { steps, .. }) => Ok(steps),
            _ => Err(ContextError::NoActiveUserTurn),
        }
    }

    fn current_open_exchange_mut(&mut self) -> Result<OpenExchangeMut<'_>, ContextError> {
        let steps = self.current_steps_mut()?;
        match steps.last_mut() {
            Some(Step::ToolExchange {
                assistant,
                tool_results,
            }) => {
                let tool_calls_len = assistant.tool_calls.as_ref().map(|v| v.len()).unwrap_or(0);

                if tool_calls_len == 0 || tool_results.len() >= tool_calls_len {
                    return Err(ContextError::ToolResultWithoutOpenExchange);
                }

                Ok(OpenExchangeMut {
                    assistant,
                    tool_results,
                })
            }
            _ => Err(ContextError::ToolResultWithoutOpenExchange),
        }
    }
}

struct OpenExchangeMut<'a> {
    assistant: &'a mut Message,
    tool_results: &'a mut Vec<Message>,
}

fn render_entry(entry: &Entry, out: &mut Vec<Message>) {
    match entry {
        Entry::Pinned { message } => out.push(message.clone()),
        Entry::UserTurn { user, steps } => {
            out.push(user.clone());
            for step in steps {
                match step {
                    Step::Assistant { message } => out.push(message.clone()),
                    Step::ToolExchange {
                        assistant,
                        tool_results,
                    } => {
                        out.push(assistant.clone());
                        out.extend(tool_results.iter().cloned());
                    }
                }
            }
        }
    }
}

fn estimate_entry_tokens(entry: &Entry, estimator: &dyn TokenEstimator) -> u32 {
    match entry {
        Entry::Pinned { message } => estimator.estimate_message_tokens(message),
        Entry::UserTurn { user, steps } => {
            let mut total = estimator.estimate_message_tokens(user);
            for step in steps {
                match step {
                    Step::Assistant { message } => {
                        total = total.saturating_add(estimator.estimate_message_tokens(message));
                    }
                    Step::ToolExchange {
                        assistant,
                        tool_results,
                    } => {
                        total = total.saturating_add(estimator.estimate_message_tokens(assistant));
                        for tool_result in tool_results {
                            total = total
                                .saturating_add(estimator.estimate_message_tokens(tool_result));
                        }
                    }
                }
            }
            total
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::llm::ToolCall;

    #[derive(Default)]
    struct LenEstimator;

    impl TokenEstimator for LenEstimator {
        fn estimate_message_tokens(&self, message: &Message) -> u32 {
            message
                .content
                .as_deref()
                .map(|s| s.len() as u32)
                .unwrap_or(0)
        }
    }

    fn msg(role: Role, content: &str) -> Message {
        Message {
            role,
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn tool_exchange_atomicity_and_order() {
        let mut ctx = Context::new();
        ctx.push_user(msg(Role::User, "u1")).unwrap();

        let assistant = Message {
            role: Role::Assistant,
            content: None,
            tool_calls: Some(vec![
                ToolCall {
                    id: "c1".to_string(),
                    function: common::llm::FunctionCall {
                        name: "t".to_string(),
                        arguments: "{}".to_string(),
                    },
                    kind: "function".to_string(),
                },
                ToolCall {
                    id: "c2".to_string(),
                    function: common::llm::FunctionCall {
                        name: "t".to_string(),
                        arguments: "{}".to_string(),
                    },
                    kind: "function".to_string(),
                },
            ]),
            tool_call_id: None,
        };

        ctx.begin_tool_exchange(assistant).unwrap();

        let mut r1 = msg(Role::Tool, "r1");
        r1.tool_call_id = Some("c1".to_string());
        ctx.push_tool_result(r1).unwrap();

        let mut r2 = msg(Role::Tool, "r2");
        r2.tool_call_id = Some("c2".to_string());
        ctx.push_tool_result(r2).unwrap();

        ctx.push_assistant(msg(Role::Assistant, "done")).unwrap();

        let linear = ctx.linear_messages();
        assert_eq!(linear.len(), 5);
        assert_eq!(linear[0].role, Role::User);
        assert_eq!(linear[1].role, Role::Assistant);
        assert_eq!(linear[2].role, Role::Tool);
        assert_eq!(linear[2].tool_call_id.as_deref(), Some("c1"));
        assert_eq!(linear[3].role, Role::Tool);
        assert_eq!(linear[3].tool_call_id.as_deref(), Some("c2"));
        assert_eq!(linear[4].content.as_deref(), Some("done"));
    }

    #[test]
    fn tool_result_requires_open_exchange() {
        let mut ctx = Context::new();
        ctx.push_user(msg(Role::User, "u1")).unwrap();

        let mut tool_result = msg(Role::Tool, "r");
        tool_result.tool_call_id = Some("c1".to_string());
        assert!(matches!(
            ctx.push_tool_result(tool_result),
            Err(ContextError::ToolResultWithoutOpenExchange)
        ));
    }

    #[test]
    fn compaction_drops_oldest_turns_preserves_pinned() {
        let mut ctx = Context::new();
        ctx.push_pinned(msg(Role::System, "ppppp"));

        ctx.push_user(msg(Role::User, "11111")).unwrap();
        ctx.push_assistant(msg(Role::Assistant, "a")).unwrap();

        ctx.push_user(msg(Role::User, "22222")).unwrap();
        ctx.push_assistant(msg(Role::Assistant, "b")).unwrap();

        ctx.push_user(msg(Role::User, "33333")).unwrap();
        ctx.push_assistant(msg(Role::Assistant, "c")).unwrap();

        let sys = vec![msg(Role::System, "sssss")];
        let limits = ContextLimits {
            max_prompt_tokens: 20,
            reserved_completion_tokens: None,
        };

        let rendered = ctx.render(&sys, Some(limits), &LenEstimator).unwrap();
        assert_eq!(rendered.dropped_turns, 2);
        assert!(rendered
            .messages
            .iter()
            .any(|m| m.content.as_deref() == Some("ppppp")));
        assert!(rendered
            .messages
            .iter()
            .any(|m| m.content.as_deref() == Some("33333")));
        assert!(!rendered
            .messages
            .iter()
            .any(|m| m.content.as_deref() == Some("11111")));
    }

    #[test]
    fn render_fails_when_system_and_pinned_exceed_budget() {
        let mut ctx = Context::new();
        ctx.push_pinned(msg(Role::System, "ppppp"));

        let sys = vec![msg(Role::System, "sssss")];
        let limits = ContextLimits {
            max_prompt_tokens: 9,
            reserved_completion_tokens: None,
        };

        assert!(matches!(
            ctx.render(&sys, Some(limits), &LenEstimator),
            Err(ContextError::PromptBudgetExceeded { .. })
        ));
    }
}
