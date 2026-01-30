"""Vector Store using Qdrant"""

from typing import List, Optional
from qdrant_client import QdrantClient
from qdrant_client.http import models
from qdrant_client.http.models import Distance, VectorParams, PointStruct
import uuid
from ..models import Memory, MemorySearchResult
from ..config import get_settings


class VectorStore:
    """Qdrant vector store for memory storage and retrieval"""
    
    def __init__(self):
        settings = get_settings()
        self.client = QdrantClient(
            host=settings.qdrant_host,
            port=settings.qdrant_port,
        )
        self.collection_name = settings.qdrant_collection
        self.dimension = settings.embedding_dimension
        self._ensure_collection()
    
    def _ensure_collection(self):
        """Ensure the collection exists"""
        collections = self.client.get_collections().collections
        exists = any(c.name == self.collection_name for c in collections)
        
        if not exists:
            self.client.create_collection(
                collection_name=self.collection_name,
                vectors_config=VectorParams(
                    size=self.dimension,
                    distance=Distance.COSINE,
                ),
            )
    
    async def add_memory(self, memory: Memory) -> str:
        """Add a memory to the vector store"""
        memory_id = memory.id or str(uuid.uuid4())
        
        point = PointStruct(
            id=memory_id,
            vector=memory.embedding,
            payload={
                "pet_id": memory.pet_id,
                "user_id": memory.user_id,
                "content": memory.content,
                "memory_type": memory.memory_type,
                "importance": memory.importance,
                "created_at": memory.created_at.isoformat(),
                "metadata": memory.metadata or {},
            },
        )
        
        self.client.upsert(
            collection_name=self.collection_name,
            points=[point],
        )
        
        return memory_id
    
    async def search(
        self,
        query_embedding: List[float],
        pet_id: int,
        user_id: str,
        top_k: int = 5,
        memory_type: Optional[str] = None,
    ) -> List[MemorySearchResult]:
        """Search for similar memories"""
        filters = [
            models.FieldCondition(
                key="pet_id",
                match=models.MatchValue(value=pet_id),
            ),
            models.FieldCondition(
                key="user_id",
                match=models.MatchValue(value=user_id),
            ),
        ]
        
        if memory_type:
            filters.append(
                models.FieldCondition(
                    key="memory_type",
                    match=models.MatchValue(value=memory_type),
                )
            )
        
        results = self.client.search(
            collection_name=self.collection_name,
            query_vector=query_embedding,
            query_filter=models.Filter(must=filters),
            limit=top_k,
        )
        
        search_results = []
        for result in results:
            memory = Memory(
                id=str(result.id),
                pet_id=result.payload["pet_id"],
                user_id=result.payload["user_id"],
                content=result.payload["content"],
                memory_type=result.payload["memory_type"],
                importance=result.payload["importance"],
                metadata=result.payload.get("metadata"),
            )
            search_results.append(MemorySearchResult(memory=memory, score=result.score))
        
        return search_results
    
    async def delete_memory(self, memory_id: str) -> bool:
        """Delete a memory by ID"""
        self.client.delete(
            collection_name=self.collection_name,
            points_selector=models.PointIdsList(points=[memory_id]),
        )
        return True
    
    async def get_memories_by_pet(
        self,
        pet_id: int,
        user_id: str,
        limit: int = 100,
    ) -> List[Memory]:
        """Get all memories for a pet"""
        results = self.client.scroll(
            collection_name=self.collection_name,
            scroll_filter=models.Filter(
                must=[
                    models.FieldCondition(
                        key="pet_id",
                        match=models.MatchValue(value=pet_id),
                    ),
                    models.FieldCondition(
                        key="user_id",
                        match=models.MatchValue(value=user_id),
                    ),
                ]
            ),
            limit=limit,
        )
        
        memories = []
        for point in results[0]:
            memory = Memory(
                id=str(point.id),
                pet_id=point.payload["pet_id"],
                user_id=point.payload["user_id"],
                content=point.payload["content"],
                memory_type=point.payload["memory_type"],
                importance=point.payload["importance"],
                metadata=point.payload.get("metadata"),
            )
            memories.append(memory)
        
        return memories
