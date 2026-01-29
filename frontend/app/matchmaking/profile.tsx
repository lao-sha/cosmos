import { TX_STATUS_TEXT, useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View
} from 'react-native';

type Gender = 'male' | 'female';

interface ProfileForm {
  nickname: string;
  age: string;
  gender: Gender;
  location: string;
  bio: string;
  birthYear: string;
  birthMonth: string;
  birthDay: string;
  birthHour: string;
}

export default function MatchmakingProfileScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { createMatchmakingProfile, isLoading, status } = useTransaction();
  
  const [form, setForm] = useState<ProfileForm>({
    nickname: '',
    age: '',
    gender: 'male',
    location: '',
    bio: '',
    birthYear: '',
    birthMonth: '',
    birthDay: '',
    birthHour: '',
  });

  const updateForm = (key: keyof ProfileForm, value: string) => {
    setForm(prev => ({ ...prev, [key]: value }));
  };

  const handleSubmit = async () => {
    if (!form.nickname || !form.age || !form.location) {
      showAlert('请填写必填项：昵称、年龄、地区');
      return;
    }

    const gender = form.gender === 'male' ? 'Male' : 'Female';
    const birthInfo = form.birthYear && form.birthMonth && form.birthDay ? {
      year: parseInt(form.birthYear),
      month: parseInt(form.birthMonth),
      day: parseInt(form.birthDay),
      hour: parseInt(form.birthHour) || 0,
    } : undefined;

    const result = await createMatchmakingProfile(form.nickname, gender, birthInfo);
    
    if (result?.success) {
      router.back();
    }
  };

  const showAlert = (msg: string) => {
    if (Platform.OS === 'web') {
      window.alert(msg);
    } else {
      Alert.alert('提示', msg);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>取消</Text>
        </Pressable>
        <Text style={styles.headerTitle}>编辑资料</Text>
        <Pressable
          style={[styles.saveButton, isLoading && styles.saveButtonDisabled]}
          onPress={handleSubmit}
          disabled={isLoading}
        >
          <Text style={styles.saveButtonText}>
            {isLoading ? (status ? TX_STATUS_TEXT[status] : '处理中...') : '保存'}
          </Text>
        </Pressable>
      </View>

      <ScrollView contentContainerStyle={styles.formContainer}>
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>
          
          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>昵称 *</Text>
            <TextInput
              style={styles.input}
              value={form.nickname}
              onChangeText={(v) => updateForm('nickname', v)}
              placeholder="请输入昵称"
              placeholderTextColor="#9ca3af"
              maxLength={20}
            />
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>年龄 *</Text>
            <TextInput
              style={styles.input}
              value={form.age}
              onChangeText={(v) => updateForm('age', v)}
              placeholder="请输入年龄"
              placeholderTextColor="#9ca3af"
              keyboardType="numeric"
              maxLength={2}
            />
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>性别 *</Text>
            <View style={styles.genderSelector}>
              <Pressable
                style={[styles.genderOption, form.gender === 'male' && styles.genderOptionSelected]}
                onPress={() => updateForm('gender', 'male')}
              >
                <Text style={[styles.genderText, form.gender === 'male' && styles.genderTextSelected]}>
                  ♂ 男
                </Text>
              </Pressable>
              <Pressable
                style={[styles.genderOption, form.gender === 'female' && styles.genderOptionSelected]}
                onPress={() => updateForm('gender', 'female')}
              >
                <Text style={[styles.genderText, form.gender === 'female' && styles.genderTextSelected]}>
                  ♀ 女
                </Text>
              </Pressable>
            </View>
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>地区 *</Text>
            <TextInput
              style={styles.input}
              value={form.location}
              onChangeText={(v) => updateForm('location', v)}
              placeholder="请输入所在城市"
              placeholderTextColor="#9ca3af"
            />
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>个人简介</Text>
            <TextInput
              style={[styles.input, styles.textArea]}
              value={form.bio}
              onChangeText={(v) => updateForm('bio', v)}
              placeholder="介绍一下自己..."
              placeholderTextColor="#9ca3af"
              multiline
              numberOfLines={4}
              maxLength={200}
            />
            <Text style={styles.charCount}>{form.bio.length}/200</Text>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>八字信息（可选）</Text>
          <Text style={styles.sectionHint}>填写后可获得更精准的八字匹配</Text>
          
          <View style={styles.birthRow}>
            <View style={styles.birthInput}>
              <Text style={styles.inputLabel}>年</Text>
              <TextInput
                style={styles.input}
                value={form.birthYear}
                onChangeText={(v) => updateForm('birthYear', v)}
                placeholder="1990"
                placeholderTextColor="#9ca3af"
                keyboardType="numeric"
                maxLength={4}
              />
            </View>
            <View style={styles.birthInput}>
              <Text style={styles.inputLabel}>月</Text>
              <TextInput
                style={styles.input}
                value={form.birthMonth}
                onChangeText={(v) => updateForm('birthMonth', v)}
                placeholder="01"
                placeholderTextColor="#9ca3af"
                keyboardType="numeric"
                maxLength={2}
              />
            </View>
            <View style={styles.birthInput}>
              <Text style={styles.inputLabel}>日</Text>
              <TextInput
                style={styles.input}
                value={form.birthDay}
                onChangeText={(v) => updateForm('birthDay', v)}
                placeholder="15"
                placeholderTextColor="#9ca3af"
                keyboardType="numeric"
                maxLength={2}
              />
            </View>
            <View style={styles.birthInput}>
              <Text style={styles.inputLabel}>时辰</Text>
              <TextInput
                style={styles.input}
                value={form.birthHour}
                onChangeText={(v) => updateForm('birthHour', v)}
                placeholder="子时"
                placeholderTextColor="#9ca3af"
              />
            </View>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>隐私设置</Text>
          <View style={styles.privacyItem}>
            <Text style={styles.privacyLabel}>八字信息仅匹配使用，不公开展示</Text>
            <Text style={styles.privacyValue}>🔒</Text>
          </View>
          <View style={styles.privacyItem}>
            <Text style={styles.privacyLabel}>资料存储在链上，您拥有完全控制权</Text>
            <Text style={styles.privacyValue}>⛓️</Text>
          </View>
        </View>

        <View style={styles.depositSection}>
          <Text style={styles.depositTitle}>💎 诚意保证金</Text>
          <Text style={styles.depositDesc}>
            缴纳保证金可提升资料可信度，违规行为将扣除保证金
          </Text>
          <Pressable style={styles.depositButton}>
            <Text style={styles.depositButtonText}>缴纳 100 STAR</Text>
          </Pressable>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 8,
  },
  backText: {
    fontSize: 15,
    color: '#6b7280',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  saveButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 8,
  },
  saveButtonDisabled: {
    backgroundColor: '#d1d5db',
  },
  saveButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  formContainer: {
    padding: 16,
    paddingBottom: 40,
  },
  section: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  sectionHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 16,
  },
  inputGroup: {
    marginTop: 16,
  },
  inputLabel: {
    fontSize: 13,
    color: '#374151',
    marginBottom: 6,
  },
  input: {
    backgroundColor: '#f9fafb',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderRadius: 8,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 15,
    color: '#1f2937',
  },
  textArea: {
    height: 100,
    textAlignVertical: 'top',
  },
  charCount: {
    fontSize: 11,
    color: '#9ca3af',
    textAlign: 'right',
    marginTop: 4,
  },
  genderSelector: {
    flexDirection: 'row',
    gap: 12,
  },
  genderOption: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#e5e7eb',
    alignItems: 'center',
    backgroundColor: '#f9fafb',
  },
  genderOptionSelected: {
    borderColor: '#6D28D9',
    backgroundColor: '#f5f3ff',
  },
  genderText: {
    fontSize: 15,
    color: '#6b7280',
  },
  genderTextSelected: {
    color: '#6D28D9',
    fontWeight: '600',
  },
  birthRow: {
    flexDirection: 'row',
    gap: 10,
  },
  birthInput: {
    flex: 1,
  },
  privacyItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  privacyLabel: {
    fontSize: 13,
    color: '#6b7280',
    flex: 1,
  },
  privacyValue: {
    fontSize: 16,
  },
  depositSection: {
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
  },
  depositTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  depositDesc: {
    fontSize: 13,
    color: '#b45309',
    textAlign: 'center',
    marginBottom: 16,
  },
  depositButton: {
    backgroundColor: '#f59e0b',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  depositButtonText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
});
