<script setup lang="ts">
import { ref, computed } from 'vue'
import { state, addEnvironment, updateEnvironment, deleteEnvironment, switchEnvironment } from '@/store'
import type { Environment } from '@/types'

// 顶部导航的环境切换已移除，此处是唯一的环境切换入口，
// 因此在标题上显式标出当前环境，避免用户不知道自己身处哪个环境
const currentEnvName = computed(
  () => state.environments.find(e => e.id === state.currentEnvId)?.name ?? '（未选择）'
)

const editing = ref<Environment | null>(null)
const isNew = ref(false)
const form = ref({
  name: '',
  description: '',
  drive_mapping: {} as Record<string, string>,
})
const newDriveLetter = ref('')
const newDrivePath = ref('')

function startNew() {
  isNew.value = true
  editing.value = null
  form.value = { name: '', description: '', drive_mapping: {} }
}

function startEdit(env: Environment) {
  isNew.value = false
  editing.value = env
  form.value = {
    name: env.name,
    description: env.description,
    drive_mapping: { ...env.drive_mapping },
  }
}

function cancelEdit() {
  editing.value = null
  isNew.value = false
}

async function save() {
  if (!form.value.name.trim()) return
  if (isNew.value) {
    const id = await addEnvironment({
      name: form.value.name.trim(),
      description: form.value.description,
      drive_mapping: form.value.drive_mapping,
      is_current: false,
    })
    // 自动切换到新环境
    await switchEnvironment(id)
  } else if (editing.value) {
    await updateEnvironment(editing.value.id, {
      name: form.value.name.trim(),
      description: form.value.description,
      drive_mapping: form.value.drive_mapping,
    })
  }
  cancelEdit()
}

async function remove(id: string) {
  if (!confirm('删除此环境？该环境下的绝对路径条目将不再显示。')) return
  await deleteEnvironment(id, false)
  if (editing.value?.id === id) cancelEdit()
}

async function set_current(id: string) {
  await switchEnvironment(id)
}

function addDriveMapping() {
  if (newDriveLetter.value && newDrivePath.value) {
    form.value.drive_mapping[newDriveLetter.value.toUpperCase()] = newDrivePath.value
    newDriveLetter.value = ''
    newDrivePath.value = ''
  }
}

function removeDriveMapping(letter: string) {
  delete form.value.drive_mapping[letter]
}
</script>

<template>
  <div class="dialog-overlay" @click.self="$emit('close')">
    <div class="dialog env-manager-dialog">
      <div class="dialog-header">
        <span>环境管理</span>
        <button class="btn btn-icon" @click="$emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <div class="env-list-section">
          <div class="section-title">
            环境列表
            <span class="env-current-hint">当前：{{ currentEnvName }}</span>
          </div>
          <div class="env-list">
            <div
              v-for="env in state.environments"
              :key="env.id"
              class="env-item"
              :class="{ active: state.currentEnvId === env.id, editing: editing?.id === env.id }"
            >
              <div class="env-info" @click="startEdit(env)">
                <span class="env-name">{{ env.name }}</span>
                <span v-if="env.is_current" class="env-current-badge">当前</span>
                <span class="env-desc">{{ env.description }}</span>
              </div>
              <div class="env-actions">
                <button
                  v-if="!env.is_current"
                  class="btn env-switch-btn"
                  title="切换到此环境"
                  @click="set_current(env.id)"
                >切换到此环境</button>
                <span v-else class="env-current-tag">使用中</span>
                <button class="btn btn-icon btn-danger" title="删除" @click="remove(env.id)">🗑</button>
              </div>
            </div>
          </div>
          <button class="btn btn-primary env-add-btn" @click="startNew">+ 新建环境</button>
        </div>

        <!-- 编辑表单 -->
        <div v-if="isNew || editing" class="env-edit-form">
          <div class="section-title">{{ isNew ? '新建环境' : '编辑环境' }}</div>
          <div class="form-row">
            <label class="form-label">名称</label>
            <input v-model="form.name" class="input" placeholder="如：家里电脑" />
          </div>
          <div class="form-row">
            <label class="form-label">描述</label>
            <input v-model="form.description" class="input" placeholder="如：Win11, 256GB SSD" />
          </div>
          <div class="form-row">
            <label class="form-label">盘符映射</label>
            <div class="drive-mappings">
              <div v-for="(path, letter) in form.drive_mapping" :key="letter" class="drive-mapping-row">
                <span class="drive-letter">{{ letter }}:</span>
                <span class="drive-path">→ {{ path }}</span>
                <button class="btn btn-icon btn-danger" @click="removeDriveMapping(letter as string)">✕</button>
              </div>
              <div class="drive-mapping-add">
                <input v-model="newDriveLetter" class="input drive-letter-input" placeholder="盘符" maxlength="1" />
                <input v-model="newDrivePath" class="input" placeholder="映射路径" />
                <button class="btn" @click="addDriveMapping">+</button>
              </div>
            </div>
          </div>
          <div class="form-actions">
            <button class="btn" @click="cancelEdit">取消</button>
            <button class="btn btn-primary" @click="save">保存</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.env-manager-dialog {
  width: 540px;
}

.env-list-section {
  margin-bottom: 20px;
}

/* 标题右侧标出当前环境 */
.env-current-hint {
  margin-left: 8px;
  font-size: 11px;
  font-weight: 400;
  color: var(--color-text-tertiary);
}

.env-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 8px;
}

.env-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  transition: all var(--transition);
}

.env-item.active {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
}

.env-item.editing {
  border-color: var(--color-primary);
}

.env-info {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  flex: 1;
}

.env-name {
  font-weight: 500;
  font-size: 13px;
}

.env-current-badge {
  font-size: 10px;
  background: var(--color-primary);
  color: white;
  padding: 1px 6px;
  border-radius: 8px;
}

.env-desc {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.env-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 切换按钮：带文字，比原来的裸箭头图标更好认 */
.env-switch-btn {
  padding: 4px 10px;
  font-size: 12px;
  white-space: nowrap;
}

/* 当前环境标记（替代原来的切换按钮位） */
.env-current-tag {
  padding: 4px 10px;
  font-size: 12px;
  color: var(--color-primary-dark);
  background: var(--color-primary-bg);
  border-radius: var(--radius-sm);
  white-space: nowrap;
}

.env-add-btn {
  width: 100%;
  justify-content: center;
}

.env-edit-form {
  border-top: 1px solid var(--color-border-light);
  padding-top: 16px;
}

.drive-mappings {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.drive-mapping-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.drive-letter {
  font-weight: 600;
  color: var(--color-primary-dark);
}

.drive-path {
  flex: 1;
  color: var(--color-text-secondary);
}

.drive-mapping-add {
  display: flex;
  gap: 4px;
}

.drive-letter-input {
  width: 60px;
  flex: 0 0 60px;
  text-transform: uppercase;
}
</style>
