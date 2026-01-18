/**
 * 擅长领域选择器组件
 */

import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Specialty, SPECIALTY_CONFIG } from '../types';

const THEME_COLOR = '#B2955D';

interface SpecialtySelectorProps {
  value: number;
  onChange: (value: number) => void;
  disabled?: boolean;
}

export function SpecialtySelector({ value, onChange, disabled }: SpecialtySelectorProps) {
  const specialties = Object.entries(SPECIALTY_CONFIG) as [string, { label: string; icon: string }][];

  const toggleSpecialty = (specialty: number) => {
    if (disabled) return;
    if (value & specialty) {
      onChange(value & ~specialty);
    } else {
      onChange(value | specialty);
    }
  };

  const isSelected = (specialty: number) => (value & specialty) !== 0;

  return (
    <View style={styles.container}>
      <Text style={styles.title}>擅长领域（至少选择1项）</Text>
      <View style={styles.grid}>
        {specialties.map(([key, config]) => {
          const specialty = Number(key);
          const selected = isSelected(specialty);
          return (
            <Pressable
              key={key}
              style={[styles.item, selected && styles.itemSelected, disabled && styles.itemDisabled]}
              onPress={() => toggleSpecialty(specialty)}
            >
              <Text style={styles.icon}>{config.icon}</Text>
              <Text style={[styles.label, selected && styles.labelSelected]}>{config.label}</Text>
            </Pressable>
          );
        })}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  title: {
    fontSize: 14,
    color: '#333',
    marginBottom: 12,
    fontWeight: '500',
  },
  grid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  item: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    backgroundColor: '#FFF',
    gap: 4,
  },
  itemSelected: {
    borderColor: THEME_COLOR,
    backgroundColor: `${THEME_COLOR}10`,
  },
  itemDisabled: {
    opacity: 0.5,
  },
  icon: {
    fontSize: 16,
  },
  label: {
    fontSize: 13,
    color: '#666',
  },
  labelSelected: {
    color: THEME_COLOR,
    fontWeight: '500',
  },
});
