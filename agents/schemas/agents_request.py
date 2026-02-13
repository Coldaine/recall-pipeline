from typing import List

from pydantic import BaseModel, Field

from agents.constants import DEFAULT_MESSAGE_TOOL, DEFAULT_MESSAGE_TOOL_KWARG
from agents.schemas.message import MessageCreate


class AgentsRequestConfig(BaseModel):
    # Flags to support the use of AssistantMessage message types
    use_assistant_message: bool = Field(
        default=True,
        description="Whether the server should parse specific tool call arguments (default `send_message`) as `AssistantMessage` objects.",
    )
    assistant_message_tool_name: str = Field(
        default=DEFAULT_MESSAGE_TOOL,
        description="The name of the designated message tool.",
    )
    assistant_message_tool_kwarg: str = Field(
        default=DEFAULT_MESSAGE_TOOL_KWARG,
        description="The name of the message argument in the designated message tool.",
    )


class AgentsRequest(BaseModel):
    messages: List[MessageCreate] = Field(
        ..., description="The messages to be sent to the agent."
    )
    config: AgentsRequestConfig = Field(
        default=AgentsRequestConfig(),
        description="Configuration options for the AgentsRequest.",
    )


class AgentsStreamingRequest(AgentsRequest):
    stream_tokens: bool = Field(
        default=False,
        description="Flag to determine if individual tokens should be streamed. Set to True for token streaming (requires stream_steps = True).",
    )
