"""Base LLM Provider Interface"""

from abc import ABC, abstractmethod
from typing import List, Optional, AsyncGenerator
from ..models import ChatMessage


class LLMProvider(ABC):
    """Abstract base class for LLM providers"""
    
    @abstractmethod
    async def chat(
        self,
        messages: List[ChatMessage],
        system_prompt: str,
        temperature: float = 0.7,
        max_tokens: int = 1000,
    ) -> str:
        """Generate a chat response"""
        pass
    
    @abstractmethod
    async def chat_stream(
        self,
        messages: List[ChatMessage],
        system_prompt: str,
        temperature: float = 0.7,
        max_tokens: int = 1000,
    ) -> AsyncGenerator[str, None]:
        """Generate a streaming chat response"""
        pass
    
    @abstractmethod
    async def health_check(self) -> bool:
        """Check if the provider is healthy"""
        pass
