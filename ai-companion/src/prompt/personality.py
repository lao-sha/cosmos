"""Personality-based Prompt Builder"""

from datetime import datetime
from typing import List, Optional
from ..models import PetInfo, Personality, Memory
from .templates import PromptTemplates


class PersonalityPromptBuilder:
    """Builds personalized prompts based on pet personality"""
    
    def __init__(self, pet_info: PetInfo):
        self.pet = pet_info
        self.personality = pet_info.personality
    
    def build_system_prompt(
        self,
        memories: Optional[List[Memory]] = None,
    ) -> str:
        """Build the complete system prompt"""
        # Calculate days together
        days_together = (datetime.utcnow() - self.pet.created_at).days
        
        # Get element traits
        element_traits = PromptTemplates.get_element_traits(self.pet.element.value)
        
        # Format memory context
        memory_context = PromptTemplates.format_memory_context(memories or [])
        
        # Build the prompt
        return PromptTemplates.SYSTEM_BASE.format(
            pet_name=self.pet.name,
            element=self._get_element_name(self.pet.element.value),
            evolution_stage=self._get_evolution_name(self.pet.evolution_stage),
            rarity=self._get_rarity_name(self.pet.rarity.value),
            level=self.pet.level,
            days_together=days_together,
            extroversion=self.personality.extroversion,
            warmth=self.personality.warmth,
            humor=self.personality.humor,
            curiosity=self.personality.curiosity,
            responsibility=self.personality.responsibility,
            element_traits=element_traits,
            memory_context=memory_context,
        )
    
    def get_speaking_style(self) -> dict:
        """Get speaking style parameters based on personality"""
        return {
            "formality": self._calc_formality(),
            "emoji_usage": self._calc_emoji_usage(),
            "sentence_length": self._calc_sentence_length(),
            "enthusiasm": self._calc_enthusiasm(),
        }
    
    def _calc_formality(self) -> str:
        """Calculate formality level"""
        # Higher warmth = less formal
        if self.personality.warmth > 70:
            return "casual"
        elif self.personality.warmth > 40:
            return "friendly"
        else:
            return "polite"
    
    def _calc_emoji_usage(self) -> str:
        """Calculate emoji usage level"""
        avg = (self.personality.extroversion + self.personality.humor) / 2
        if avg > 70:
            return "high"
        elif avg > 40:
            return "medium"
        else:
            return "low"
    
    def _calc_sentence_length(self) -> str:
        """Calculate preferred sentence length"""
        if self.personality.extroversion > 70:
            return "long"
        elif self.personality.extroversion > 40:
            return "medium"
        else:
            return "short"
    
    def _calc_enthusiasm(self) -> str:
        """Calculate enthusiasm level"""
        avg = (self.personality.extroversion + self.personality.warmth) / 2
        if avg > 70:
            return "high"
        elif avg > 40:
            return "medium"
        else:
            return "reserved"
    
    def _get_element_name(self, element: str) -> str:
        """Get Chinese element name"""
        names = {
            "normal": "普通",
            "fire": "火焰",
            "water": "水",
            "shadow": "暗影",
            "light": "光明",
        }
        return names.get(element, "普通")
    
    def _get_evolution_name(self, stage: int) -> str:
        """Get evolution stage name"""
        names = {
            0: "幼年期",
            1: "成长期",
            2: "成熟期",
            3: "完全体",
            4: "究极体",
        }
        return names.get(stage, "幼年期")
    
    def _get_rarity_name(self, rarity: str) -> str:
        """Get Chinese rarity name"""
        names = {
            "common": "普通",
            "rare": "稀有",
            "epic": "史诗",
            "legendary": "传说",
            "mythic": "神话",
        }
        return names.get(rarity, "普通")
    
    def should_initiate_conversation(self) -> bool:
        """Determine if pet should initiate conversation"""
        # Higher extroversion = more likely to initiate
        import random
        threshold = self.personality.extroversion / 100
        return random.random() < threshold * 0.3
    
    def get_greeting(self) -> str:
        """Get appropriate greeting based on time"""
        hour = datetime.now().hour
        if 5 <= hour < 12:
            return PromptTemplates.GREETING_MORNING.format(pet_name=self.pet.name)
        elif 18 <= hour < 24:
            return PromptTemplates.GREETING_EVENING.format(pet_name=self.pet.name)
        else:
            return f"{self.pet.name}在这里喵~ 有什么想聊的吗？"
