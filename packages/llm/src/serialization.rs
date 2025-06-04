use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

use crate::bindings::exports::wavs::agent::errors::AgentError;
use crate::bindings::exports::wavs::agent::types::{
    ContractCall, Function, Message, Tool, ToolCall, ToolCallFunction, Transaction,
};

// Implement From<String> for AgentError to handle map_err with ?
impl From<String> for AgentError {
    fn from(error: String) -> Self {
        AgentError::Other(error)
    }
}

// Serialization for Message
#[derive(Serialize, Deserialize)]
struct MessageJson {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCallJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json = MessageJson {
            role: self.role.clone(),
            content: self.content.clone(),
            tool_calls: self
                .tool_calls
                .as_ref()
                .map(|calls| calls.iter().map(|call| ToolCallJson::from(call)).collect()),
            tool_call_id: self.tool_call_id.clone(),
            name: self.name.clone(),
        };
        json.serialize(serializer)
    }
}

struct MessageVisitor {
    marker: PhantomData<fn() -> Message>,
}

impl MessageVisitor {
    fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<'de> Visitor<'de> for MessageVisitor {
    type Value = Message;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid message object")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Message, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut role = None;
        let mut content = None;
        let mut tool_calls = None;
        let mut tool_call_id = None;
        let mut name = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "role" => role = Some(map.next_value()?),
                "content" => content = Some(map.next_value()?),
                "tool_calls" => {
                    let json_calls: Vec<ToolCallJson> = map.next_value()?;
                    tool_calls = Some(json_calls.into_iter().map(|call| call.into()).collect());
                }
                "tool_call_id" => tool_call_id = Some(map.next_value()?),
                "name" => name = Some(map.next_value()?),
                _ => {
                    let _: serde_json::Value = map.next_value()?;
                }
            }
        }

        let role = role.ok_or_else(|| Error::missing_field("role"))?;

        Ok(Message { role, content, tool_calls, tool_call_id, name })
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MessageVisitor::new())
    }
}

// Serialization for Tool
#[derive(Serialize, Deserialize)]
struct ToolJson {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionJson,
}

impl From<&Tool> for ToolJson {
    fn from(tool: &Tool) -> Self {
        ToolJson { tool_type: tool.tool_type.clone(), function: FunctionJson::from(&tool.function) }
    }
}

impl Serialize for Tool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ToolJson::from(self).serialize(serializer)
    }
}

// Serialization for Function
#[derive(Serialize, Deserialize)]
struct FunctionJson {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

impl From<&Function> for FunctionJson {
    fn from(function: &Function) -> Self {
        let parameters = function.parameters.as_ref().and_then(|p| serde_json::from_str(p).ok());

        FunctionJson {
            name: function.name.clone(),
            description: function.description.clone(),
            parameters,
        }
    }
}

// Serialization for ToolCall
#[derive(Serialize, Deserialize)]
struct ToolCallJson {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolCallFunctionJson,
}

impl From<&ToolCall> for ToolCallJson {
    fn from(call: &ToolCall) -> Self {
        ToolCallJson {
            id: call.id.clone(),
            tool_type: call.tool_type.clone(),
            function: ToolCallFunctionJson::from(&call.function),
        }
    }
}

impl From<ToolCallJson> for ToolCall {
    fn from(json: ToolCallJson) -> Self {
        ToolCall { id: json.id, tool_type: json.tool_type, function: json.function.into() }
    }
}

// Serialization for ToolCallFunction
#[derive(Serialize, Deserialize)]
struct ToolCallFunctionJson {
    name: String,
    arguments: String,
}

impl From<&ToolCallFunction> for ToolCallFunctionJson {
    fn from(function: &ToolCallFunction) -> Self {
        ToolCallFunctionJson { name: function.name.clone(), arguments: function.arguments.clone() }
    }
}

impl From<ToolCallFunctionJson> for ToolCallFunction {
    fn from(json: ToolCallFunctionJson) -> Self {
        ToolCallFunction { name: json.name, arguments: json.arguments }
    }
}

// Serialization for Transaction
impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serde_json::Map::new();
        map.insert("to".into(), serde_json::Value::String(self.to.clone()));
        map.insert("value".into(), serde_json::Value::String(self.value.clone()));
        map.insert("data".into(), serde_json::Value::String(self.data.clone()));
        map.insert("description".into(), serde_json::Value::String(self.description.clone()));

        if let Some(contract_call) = &self.contract_call {
            let mut call_map = serde_json::Map::new();
            call_map.insert(
                "function".into(),
                serde_json::Value::String(contract_call.function.clone()),
            );

            let args = contract_call
                .args
                .iter()
                .map(|arg| serde_json::Value::String(arg.clone()))
                .collect();

            call_map.insert("args".into(), serde_json::Value::Array(args));
            map.insert("contract_call".into(), serde_json::Value::Object(call_map));
        }

        serde_json::Value::Object(map).serialize(serializer)
    }
}

// Deserialization for Transaction
impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TransactionJson {
            to: String,
            value: String,
            #[serde(default)]
            data: String,
            #[serde(default)]
            description: String,
            contract_call: Option<ContractCallJson>,
        }

        #[derive(Deserialize)]
        struct ContractCallJson {
            function: String,
            args: Vec<String>,
        }

        let json = TransactionJson::deserialize(deserializer)?;

        let contract_call = json
            .contract_call
            .map(|call| ContractCall { function: call.function, args: call.args });

        Ok(Transaction {
            to: json.to,
            value: json.value,
            data: json.data,
            description: json.description,
            contract_call,
        })
    }
}
