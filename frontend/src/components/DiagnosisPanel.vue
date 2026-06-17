<template>
  <div class="diagnosis-panel">
    <div v-if="!hasResult" class="empty-state">
      <div class="icon">
        <el-icon :size="64"><first-aid-kit /></el-icon>
      </div>
      <p>请先运行一次仿真，再进行工况诊断</p>
    </div>

    <div v-else>
      <div class="diagnosis-actions">
        <el-button
          type="primary"
          size="large"
          class="btn-primary"
          :loading="diagnosing"
          @click="startDiagnosis"
        >
          <el-icon style="margin-right: 6px"><search /></el-icon>
          {{ diagnosing ? '诊断分析中...' : '开始诊断' }}
        </el-button>
      </div>

      <div v-if="diagnosisResult && diagnosisResult.conclusions.length > 0" class="diagnosis-results">
        <div class="diagnosis-summary">
          <el-alert
            type="warning"
            :closable="false"
            show-icon
          >
            <template #title>
              诊断发现 <b>{{ diagnosisResult.conclusions.length }}</b> 条工况问题，请查看下方详细诊断结论
            </template>
          </el-alert>
        </div>

        <div class="diagnosis-cards">
          <div
            v-for="(item, index) in diagnosisResult.conclusions"
            :key="index"
            class="diagnosis-card"
            :class="'severity-' + severityClass(item.severity)"
          >
            <div class="card-header">
              <div class="card-category">
                <el-icon :size="20"><component :is="categoryIcon(item.category)" /></el-icon>
                <span class="category-label">{{ item.category }}</span>
              </div>
              <el-tag
                :type="severityTagType(item.severity)"
                size="default"
                effect="dark"
              >
                {{ item.severity }}
              </el-tag>
            </div>
            <div class="card-description">{{ item.description }}</div>
            <div class="card-params">
              <div class="param-row">
                <span class="param-label">关联参数:</span>
                <span class="param-name">{{ item.related_param }}</span>
              </div>
              <div class="param-compare">
                <div class="param-value current">
                  <div class="param-value-label">当前值</div>
                  <div class="param-value-number">{{ formatValue(item.current_value) }}</div>
                </div>
                <div class="param-arrow">
                  <el-icon :size="20"><right /></el-icon>
                </div>
                <div class="param-value recommended">
                  <div class="param-value-label">推荐值</div>
                  <div class="param-value-number">{{ formatValue(item.recommended_value) }}</div>
                </div>
              </div>
            </div>
            <div class="card-detail">
              <el-collapse>
                <el-collapse-item title="推荐算法说明">
                  <p>{{ item.recommendation_detail }}</p>
                </el-collapse-item>
              </el-collapse>
            </div>
          </div>
        </div>

        <div class="apply-actions">
          <el-button
            type="success"
            size="large"
            @click="applyRecommendations"
            :disabled="applying"
            :loading="applying"
          >
            <el-icon style="margin-right: 6px"><check /></el-icon>
            一键应用推荐参数
          </el-button>
          <el-button size="large" @click="clearDiagnosis">
            清除诊断结果
          </el-button>
        </div>
      </div>

      <div v-else-if="diagnosisResult && diagnosisResult.conclusions.length === 0" class="diagnosis-results">
        <el-result icon="success" title="系统工况正常" sub-title="当前仿真结果未发现明显工况问题，各项指标均在合理范围内">
          <template #extra>
            <el-button type="primary" @click="clearDiagnosis">确定</el-button>
          </template>
        </el-result>
      </div>

      <div class="diagnosis-history" v-if="historyList.length > 0">
        <div class="history-title">
          <el-icon><clock /></el-icon>
          诊断历史记录
        </div>
        <div class="history-list">
          <div
            v-for="(record, index) in historyList"
            :key="record.id"
            class="history-item"
            :class="{ active: activeHistoryId === record.id }"
            @click="viewHistory(record)"
          >
            <div class="history-info">
              <div class="history-time">{{ record.timestamp }}</div>
              <div class="history-summary">
                发现 <b>{{ record.conclusions.length }}</b> 条问题
                <el-tag
                  v-if="getMaxSeverity(record.conclusions)"
                  :type="severityTagType(getMaxSeverity(record.conclusions))"
                  size="small"
                  style="margin-left: 6px"
                >
                  {{ getMaxSeverity(record.conclusions) }}
                </el-tag>
              </div>
            </div>
            <el-button
              size="small"
              type="danger"
              link
              @click.stop="deleteHistory(record.id)"
            >
              <el-icon><delete /></el-icon>
            </el-button>
          </div>
        </div>

        <el-dialog
          v-model="historyDialogVisible"
          title="历史诊断记录 (只读)"
          width="700px"
          :close-on-click-modal="true"
        >
          <div v-if="activeHistoryRecord" class="history-detail">
            <div class="history-detail-meta">
              <span>诊断时间: {{ activeHistoryRecord.timestamp }}</span>
              <el-tag type="info" size="small">只读</el-tag>
            </div>
            <div class="diagnosis-cards">
              <div
                v-for="(item, idx) in activeHistoryRecord.conclusions"
                :key="idx"
                class="diagnosis-card"
                :class="'severity-' + severityClass(item.severity)"
              >
                <div class="card-header">
                  <div class="card-category">
                    <el-icon :size="20"><component :is="categoryIcon(item.category)" /></el-icon>
                    <span class="category-label">{{ item.category }}</span>
                  </div>
                  <el-tag :type="severityTagType(item.severity)" size="default" effect="dark">
                    {{ item.severity }}
                  </el-tag>
                </div>
                <div class="card-description">{{ item.description }}</div>
                <div class="card-params">
                  <div class="param-row">
                    <span class="param-label">关联参数:</span>
                    <span class="param-name">{{ item.related_param }}</span>
                  </div>
                  <div class="param-compare">
                    <div class="param-value current">
                      <div class="param-value-label">当时值</div>
                      <div class="param-value-number">{{ formatValue(item.current_value) }}</div>
                    </div>
                    <div class="param-arrow">
                      <el-icon :size="20"><right /></el-icon>
                    </div>
                    <div class="param-value recommended">
                      <div class="param-value-label">推荐值</div>
                      <div class="param-value-number">{{ formatValue(item.recommended_value) }}</div>
                    </div>
                  </div>
                </div>
                <div class="card-detail">
                  <p class="detail-text">{{ item.recommendation_detail }}</p>
                </div>
              </div>
            </div>
          </div>
          <template #footer>
            <el-button @click="historyDialogVisible = false">关闭</el-button>
          </template>
        </el-dialog>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { simulationApi } from '../api'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Search, Check, Delete, Clock, Right,
  FirstAidKit, TrendCharts, Cpu, Sunny, SetUp
} from '@element-plus/icons-vue'

const props = defineProps({
  simulationResult: {
    type: Object,
    default: null
  },
  influent: {
    type: Object,
    required: true
  },
  processConfig: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['apply-params', 'run-simulation'])

const diagnosing = ref(false)
const applying = ref(false)
const diagnosisResult = ref(null)
const historyList = ref([])
const historyDialogVisible = ref(false)
const activeHistoryId = ref(null)
const activeHistoryRecord = ref(null)

const HISTORY_KEY = 'wastewater_sim_diagnosis_history'

const hasResult = computed(() => !!props.simulationResult)

const severityClass = (severity) => {
  if (severity === '轻度') return 'mild'
  if (severity === '中度') return 'moderate'
  if (severity === '重度') return 'severe'
  return 'mild'
}

const severityTagType = (severity) => {
  if (severity === '轻度') return 'warning'
  if (severity === '中度') return 'danger'
  if (severity === '重度') return 'danger'
  return 'info'
}

const categoryIcon = (category) => {
  const map = {
    '硝化': TrendCharts,
    '反硝化': Cpu,
    '除磷': Sunny,
    '有机物降解': SetUp,
    '固液分离': FirstAidKit
  }
  return map[category] || FirstAidKit
}

const formatValue = (val) => {
  if (val === null || val === undefined) return '-'
  if (Number.isInteger(val)) return val.toString()
  if (Math.abs(val) >= 100) return val.toFixed(1)
  if (Math.abs(val) >= 1) return val.toFixed(2)
  return val.toFixed(3)
}

const getMaxSeverity = (conclusions) => {
  if (!conclusions || conclusions.length === 0) return null
  const order = { '重度': 3, '中度': 2, '轻度': 1 }
  let max = 0
  let result = null
  for (const c of conclusions) {
    if (order[c.severity] > max) {
      max = order[c.severity]
      result = c.severity
    }
  }
  return result
}

const loadHistory = () => {
  try {
    const stored = localStorage.getItem(HISTORY_KEY)
    if (stored) {
      historyList.value = JSON.parse(stored)
    }
  } catch (e) {
    historyList.value = []
  }
}

const saveHistory = () => {
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(historyList.value))
  } catch (e) {
    console.warn('保存诊断历史失败', e)
  }
}

const addHistoryRecord = (conclusions, inputSnapshot) => {
  const now = new Date()
  const pad = n => String(n).padStart(2, '0')
  const timestamp = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`

  const record = {
    id: Date.now().toString(36) + Math.random().toString(36).substr(2, 5),
    timestamp,
    conclusions: JSON.parse(JSON.stringify(conclusions)),
    inputSnapshot: JSON.parse(JSON.stringify(inputSnapshot))
  }

  historyList.value.unshift(record)
  if (historyList.value.length > 10) {
    historyList.value = historyList.value.slice(0, 10)
  }
  saveHistory()
}

const deleteHistory = (id) => {
  historyList.value = historyList.value.filter(h => h.id !== id)
  saveHistory()
}

const viewHistory = (record) => {
  activeHistoryId.value = record.id
  activeHistoryRecord.value = record
  historyDialogVisible.value = true
}

const startDiagnosis = async () => {
  if (!props.simulationResult) {
    ElMessage.warning('请先运行一次仿真')
    return
  }

  diagnosing.value = true
  diagnosisResult.value = null

  try {
    const reqData = {
      influent: { ...props.influent },
      process_config: JSON.parse(JSON.stringify(props.processConfig)),
      final_effluent: props.simulationResult.final_effluent,
      unit_effluents: props.simulationResult.unit_effluents,
      compliance: props.simulationResult.compliance
    }

    const res = await simulationApi.diagnose(reqData)

    if (res.success && res.data) {
      diagnosisResult.value = res.data
      addHistoryRecord(res.data.conclusions, {
        influent: { ...props.influent },
        processConfig: JSON.parse(JSON.stringify(props.processConfig)),
        compliance: props.simulationResult.compliance
      })

      if (res.data.conclusions.length > 0) {
        ElMessage.success(`诊断完成，发现${res.data.conclusions.length}条工况问题`)
      } else {
        ElMessage.success('诊断完成，系统工况正常')
      }
    } else {
      ElMessage.error(res.message || '诊断失败')
    }
  } catch (e) {
    ElMessage.error('诊断请求失败: ' + (e.message || '未知错误'))
  } finally {
    diagnosing.value = false
  }
}

const applyRecommendations = () => {
  if (!diagnosisResult.value || diagnosisResult.value.conclusions.length === 0) return

  ElMessageBox.confirm(
    '将用推荐值覆盖当前参数配置，并自动运行一次仿真。是否继续？',
    '确认应用推荐参数',
    {
      type: 'warning',
      confirmButtonText: '应用并仿真',
      cancelButtonText: '取消'
    }
  ).then(() => {
    applying.value = true

    const updatedConfig = JSON.parse(JSON.stringify(props.processConfig))

    for (const conclusion of diagnosisResult.value.conclusions) {
      const path = conclusion.param_path
      const value = conclusion.recommended_value

      if (path === 'aeration_rate') {
        updatedConfig.aeration_rate = value
      } else if (path === 'internal_recirculation_ratio') {
        updatedConfig.internal_recirculation_ratio = value
      } else if (path === 'srt') {
        updatedConfig.srt = value
      } else if (path === 'anoxic.do_setpoint') {
        updatedConfig.anoxic.do_setpoint = value
      } else if (path === 'anaerobic.do_setpoint') {
        updatedConfig.anaerobic.do_setpoint = value
      } else if (path === 'anaerobic.hrt') {
        updatedConfig.anaerobic.hrt = value
      } else if (path === 'clarifier.hrt') {
        updatedConfig.clarifier.hrt = value
      }
    }

    emit('apply-params', updatedConfig)
    applying.value = false
  }).catch(() => {
    applying.value = false
  })
}

const clearDiagnosis = () => {
  diagnosisResult.value = null
}

loadHistory()
</script>

<style scoped>
.diagnosis-panel {
  width: 100%;
}

.diagnosis-actions {
  margin-bottom: 20px;
  display: flex;
  justify-content: center;
}

.diagnosis-actions .el-button {
  width: 240px;
}

.diagnosis-summary {
  margin-bottom: 20px;
}

.diagnosis-cards {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 24px;
}

.diagnosis-card {
  border-radius: 10px;
  padding: 20px;
  border: 1px solid #ebeef5;
  transition: all 0.3s;
  background: white;
}

.diagnosis-card:hover {
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}

.diagnosis-card.severity-mild {
  border-left: 4px solid #e6a23c;
}

.diagnosis-card.severity-moderate {
  border-left: 4px solid #f56c6c;
  background: #fff8f8;
}

.diagnosis-card.severity-severe {
  border-left: 4px solid #f56c6c;
  background: #fef0f0;
  box-shadow: 0 2px 8px rgba(245, 108, 108, 0.15);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.card-category {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.category-label {
  font-weight: 700;
}

.card-description {
  font-size: 15px;
  color: #606266;
  margin-bottom: 16px;
  line-height: 1.6;
}

.card-params {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 14px 16px;
  margin-bottom: 8px;
}

.param-row {
  margin-bottom: 12px;
  font-size: 13px;
}

.param-label {
  color: #909399;
  margin-right: 6px;
}

.param-name {
  color: #303133;
  font-weight: 600;
}

.param-compare {
  display: flex;
  align-items: center;
  gap: 12px;
}

.param-value {
  flex: 1;
  text-align: center;
  padding: 10px 12px;
  border-radius: 8px;
}

.param-value.current {
  background: #fef0f0;
  border: 1px solid #fde2e2;
}

.param-value.recommended {
  background: #f0f9eb;
  border: 1px solid #e1f3d8;
}

.param-value-label {
  font-size: 12px;
  color: #909399;
  margin-bottom: 4px;
}

.param-value-number {
  font-size: 20px;
  font-weight: 700;
}

.param-value.current .param-value-number {
  color: #f56c6c;
}

.param-value.recommended .param-value-number {
  color: #67c23a;
}

.param-arrow {
  color: #c0c4cc;
  flex-shrink: 0;
}

.card-detail {
  margin-top: 4px;
}

.card-detail :deep(.el-collapse-item__header) {
  font-size: 13px;
  color: #409eff;
  height: 36px;
  line-height: 36px;
}

.card-detail :deep(.el-collapse-item__wrap) {
  border-bottom: none;
}

.card-detail :deep(.el-collapse-item__content) {
  font-size: 13px;
  color: #909399;
  line-height: 1.8;
  padding-bottom: 4px;
}

.card-detail .detail-text {
  font-size: 13px;
  color: #909399;
  line-height: 1.8;
  padding: 4px 0;
}

.apply-actions {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin: 24px 0;
  padding: 20px;
  background: #f5f7fa;
  border-radius: 10px;
}

.diagnosis-history {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid #ebeef5;
}

.history-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: #fafafa;
  border-radius: 8px;
  border: 1px solid #ebeef5;
  cursor: pointer;
  transition: all 0.2s;
}

.history-item:hover {
  background: #f5f7fa;
  border-color: #dcdfe6;
}

.history-item.active {
  border-color: #409eff;
  background: #ecf5ff;
}

.history-info {
  flex: 1;
}

.history-time {
  font-size: 13px;
  color: #909399;
  margin-bottom: 4px;
}

.history-summary {
  font-size: 14px;
  color: #606266;
}

.history-detail-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding: 10px 16px;
  background: #f5f7fa;
  border-radius: 6px;
  font-size: 13px;
  color: #909399;
}
</style>
