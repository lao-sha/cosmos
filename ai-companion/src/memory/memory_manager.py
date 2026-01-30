"""Memory Manager - Manages short-term and long-term memory"""

from typing import List, Optional
from datetime import datetime
from .embedding import EmbeddingService
from .vector_store import VectorStore
from ..models import Memory, MemorySearchResult, ChatMessage
from ..config import get_settings


class MemoryManager:
    """Manages pet memories for personalized conversations"""
    
    def __init__(self):
        self.embedding_service = EmbeddingService()
        self.vector_store = VectorStore()
        self.settings = get_settings()
        # Short-term memory cache (in-memory)
        self._short_term: dict[str, List[ChatMessage]] = {}
    
    def _get_cache_key(self, pet_id: int, user_id: str) -> str:
        """Generate cache key for short-term memory"""
        return f"{pet_id}:{user_id}"
    
    def add_to_short_term(
        self,
        pet_id: int,
        user_id: str,
        message: ChatMessage,
    ):
        """Add a message to short-term memory"""
        key = self._get_cache_key(pet_id, user_id)
        if key not in self._short_term:
            self._short_term[key] = []
        
        self._short_term[key].append(message)
        
        # Trim to max size
        max_size = self.settings.max_short_term_memory
        if len(self._short_term[key]) > max_size:
            self._short_term[key] = self._short_term[key][-max_size:]
    
    def get_short_term(
        self,
        pet_id: int,
        user_id: str,
        limit: Optional[int] = None,
    ) -> List[ChatMessage]:
        """Get short-term memory (recent conversation)"""
        key = self._get_cache_key(pet_id, user_id)
        messages = self._short_term.get(key, [])
        if limit:
            return messages[-limit:]
        return messages
    
    def clear_short_term(self, pet_id: int, user_id: str):
        """Clear short-term memory"""
        key = self._get_cache_key(pet_id, user_id)
        self._short_term.pop(key, None)
    
    async def save_to_long_term(
        self,
        pet_id: int,
        user_id: str,
        content: str,
        memory_type: str = "conversation",
        importance: float = 0.5,
        metadata: Optional[dict] = None,
    ) -> str:
        """Save a memory to long-term storage (vector DB)"""
        # Generate embedding
        embedding = self.embedding_service.embed(content)
        
        memory = Memory(
            pet_id=pet_id,
            user_id=user_id,
            content=content,
            memory_type=memory_type,
            importance=importance,
            embedding=embedding,
            created_at=datetime.utcnow(),
            metadata=metadata,
        )
        
        return await self.vector_store.add_memory(memory)
    
    async def retrieve_relevant_memories(
        self,
        pet_id: int,
        user_id: str,
        query: str,
        top_k: Optional[int] = None,
        memory_type: Optional[str] = None,
    ) -> List[MemorySearchResult]:
        """Retrieve relevant memories based on query"""
        # Generate query embedding
        query_embedding = self.embedding_service.embed(query)
        
        k = top_k or self.settings.memory_retrieval_top_k
        
        return await self.vector_store.search(
            query_embedding=query_embedding,
            pet_id=pet_id,
            user_id=user_id,
            top_k=k,
            memory_type=memory_type,
        )
    
    async def get_context_for_chat(
        self,
        pet_id: int,
        user_id: str,
        current_message: str,
    ) -> dict:
        """Get full context for chat including short-term and relevant long-term memories"""
        # Get short-term memory
        short_term = self.get_short_term(pet_id, user_id)
        
        # Retrieve relevant long-term memories
        relevant_memories = await self.retrieve_relevant_memories(
            pet_id=pet_id,
            user_id=user_id,
            query=current_message,
        )
        
        return {
            "short_term": short_term,
            "long_term": [r.memory for r in relevant_memories],
            "long_term_scores": [r.score for r in relevant_memories],
        }
    
    async def summarize_and_store(
        self,
        pet_id: int,
        user_id: str,
        messages: List[ChatMessage],
    ) -> Optional[str]:
        """Summarize a conversation and store as long-term memory"""
        if len(messages) < 4:
            return None
        
        # Create a simple summary (in production, use LLM for better summary)
        user_messages = [m.content for m in messages if m.role == "user"]
        summary = f"对话主题: {user_messages[0][:50]}... 共{len(messages)}条消息"
        
        return await self.save_to_long_term(
            pet_id=pet_id,
            user_id=user_id,
            content=summary,
            memory_type="conversation_summary",
            importance=0.6,
            metadata={"message_count": len(messages)},
        )
    
    async def delete_memory(self, memory_id: str) -> bool:
        """Delete a specific memory"""
        return await self.vector_store.delete_memory(memory_id)
    
    async def get_all_memories(
        self,
        pet_id: int,
        user_id: str,
        limit: int = 100,
    ) -> List[Memory]:
        """Get all memories for a pet"""
        return await self.vector_store.get_memories_by_pet(pet_id, user_id, limit)
