<template>
  <div class="diagnosis-panel">
    <div v-if="!hasResult" class="empty-state">
      <div class="icon">
        <el-icon :size="64"><first-aid-kit /></el-icon>
      </div>
      <p>请先运行一次仿真，再进行工况诊断</p>
    </div>

    <div v-else>
      <div class="diagnosis-toolbar">
        <div class="toolbar-left">
          <el-switch
            v-model="compareMode"
            active-text="对比上次诊断"
            :disabled="historyList.length < 1 || diagnosing"
            style="--el-switch-on-color: #409eff"
          />
          <el-tooltip content="将当前诊断与上一次诊断结果并排对比" placement="top" v-if="historyList.length < 1">
            <span class="switch-hint">（需至少有1条历史记录）</span>
          </el-tooltip>
        </div>
        <div class="toolbar-right">
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
      </div>

      <div v-if="diagnosisResult && diagnosisResult.conclusions.length > 0" class="diagnosis-results">
        <div class="diagnosis-summary">
          <el-alert
            type="warning"
            :closable="false"
            show-icon
          >
            <template #title>
              <span v-if="!compareMode">诊断发现 <b>{{ diagnosisResult.conclusions.length }}</b> 条工况问题，请查看下方详细诊断结论</span>
              <span v-else>
                当前诊断 <b>{{ diagnosisResult.conclusions.length }}</b> 条问题，
                上次诊断 <b>{{ lastDiagnosis ? lastDiagnosis.conclusions.length : 0 }}</b> 条问题，
                对比模式已开启
              </span>
            </template>
          </el-alert>
        </div>

        <div class="diagnosis-cards" v-if="!compareMode">
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
              <div class="card-actions">
                <el-button
                  size="small"
                  type="primary"
                  link
                  @click="toggleTimeSeries(index)"
                >
                  <el-icon><component :is="expandedTimeSeries[index] ? 'caret-top' : 'caret-bottom'" /></el-icon>
                  <span style="margin-left: 2px">{{ expandedTimeSeries[index] ? '收起时序' : '查看关联时序' }}</span>
                </el-button>
                <el-tag
                  :color="severityColor(item.severity)"
                  size="default"
                  effect="dark"
                  style="border: none; margin-left: 8px"
                >
                  {{ item.severity }}
                </el-tag>
              </div>
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

            <div v-if="expandedTimeSeries[index]" class="card-timeseries">
              <div class="timeseries-title">
                <el-icon><trend-charts /></el-icon>
                <span>出水 {{ getIndicatorLabel(item.category) }} 时序变化</span>
                <el-tag type="info" size="small" style="margin-left: 8px">
                  达标限值: {{ getComplianceLimit(item.category) }} mg/L
                </el-tag>
              </div>
              <v-chart
                class="timeseries-chart"
                :option="buildTimeSeriesOption(item, index)"
                autoresize
                :key="'ts-' + index"
                ref="tsChartRefs"
              />
            </div>

            <div class="card-detail">
              <el-collapse v-model="expandedDetails">
                <el-collapse-item :name="'detail-' + index">
                  <template #title>
                    <span class="collapse-title">
                      <el-icon><document /></el-icon>
                      推荐算法说明
                    </span>
                  </template>
                  <p>{{ item.recommendation_detail }}</p>
                  <div class="estimate-row" style="margin-top: 10px; padding-top: 10px; border-top: 1px dashed #ebeef5">
                    <span class="estimate-label">
                      <el-icon><aim /></el-icon>
                      预估影响:
                    </span>
                    <span class="estimate-value" :class="estimateStatusClass(index)">
                      <span v-if="estimateLoading[index]" class="estimate-loading">
                        <el-icon class="is-loading"><loading /></el-icon>
                        正在估算...
                      </span>
                      <span v-else-if="estimateResult[index] !== null && estimateResult[index] !== undefined">
                        预计出水{{ getIndicatorLabel(item.category) }}
                        <b :class="estimateTrendClass(estimateResult[index])">
                          {{ estimateResult[index] >= 0 ? '上升' : '下降' }}
                          {{ Math.abs(estimateResult[index]).toFixed(1) }}%
                        </b>
                        （从 {{ formatValue(getCurrentIndicatorValue(item.category)) }} mg/L
                        → {{ formatValue(estimateResultAbsolute[index]) }} mg/L）
                      </span>
                      <span v-else class="estimate-na">无法预估</span>
                    </span>
                  </div>
                </el-collapse-item>
              </el-collapse>
            </div>
          </div>
        </div>

        <div class="diagnosis-cards compare-mode" v-else>
          <div
            v-for="(pair, pIndex) in mergedComparisonList"
            :key="'pair-' + pIndex"
            class="comparison-card-wrapper"
          >
            <div class="comparison-card-titlebar">
              <span class="comparison-title-label">
                <el-icon><component :is="categoryIcon(pair.category)" /></el-icon>
                {{ pair.category }} - {{ pair.description }}
              </span>
              <div class="comparison-title-badges">
                <el-tag
                  v-if="pair.current"
                  :color="severityColor(pair.current.severity)"
                  size="small"
                  effect="dark"
                  style="border: none; margin-right: 6px"
                >
                  当前: {{ pair.current.severity }}
                </el-tag>
                <el-tag
                  v-if="pair.last"
                  :color="severityColor(pair.last.severity)"
                  size="small"
                  effect="dark"
                  style="border: none"
                >
                  上次: {{ pair.last.severity }}
                </el-tag>
              </div>
            </div>

            <div class="comparison-columns">
              <div class="comparison-col comparison-current">
                <div class="col-header">
                  <el-tag type="primary" size="small" effect="light">当前诊断</el-tag>
                </div>
                <div v-if="pair.current" class="col-body" :class="'severity-' + severityClass(pair.current.severity)">
                  <div class="col-description">{{ pair.current.description }}</div>
                  <div class="col-param-row">
                    <span class="col-param-label">关联参数:</span>
                    <span class="col-param-name">{{ pair.current.related_param }}</span>
                  </div>
                  <div class="col-param-compare">
                    <div class="col-param-block">
                      <div class="col-param-label-sm">当前值</div>
                      <div class="col-param-value current-v">{{ formatValue(pair.current.current_value) }}</div>
                    </div>
                    <div class="col-arrow">
                      <el-icon :size="18"><right /></el-icon>
                    </div>
                    <div class="col-param-block">
                      <div class="col-param-label-sm">推荐值</div>
                      <div class="col-param-value recommended-v">
                        {{ formatValue(pair.current.recommended_value) }}
                        <span v-if="pair.last" class="trend-arrow" :class="getCompareTrendClass(pair)">
                          <el-icon><component :is="getCompareTrendIcon(pair)" /></el-icon>
                          {{ getCompareTrendText(pair) }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
                <div v-else class="col-body col-empty">
                  <div class="empty-icon-wrap">
                    <el-icon :size="32" color="#67c23a"><circle-check /></el-icon>
                  </div>
                  <div class="empty-text">当前无此问题</div>
                  <div class="col-param-row placeholder-row">
                    <span class="col-param-label">关联参数:</span>
                    <span class="col-param-name placeholder">—</span>
                  </div>
                  <div class="col-param-compare">
                    <div class="col-param-block placeholder-block">
                      <div class="col-param-label-sm">当前值</div>
                      <div class="col-param-value placeholder-v">—</div>
                    </div>
                    <div class="col-arrow placeholder-arrow">
                      <el-icon :size="18"><right /></el-icon>
                    </div>
                    <div class="col-param-block placeholder-block">
                      <div class="col-param-label-sm">推荐值</div>
                      <div class="col-param-value placeholder-v">—</div>
                    </div>
                  </div>
                </div>
              </div>

              <div class="comparison-divider">
                <div class="divider-line"></div>
                <el-icon :size="24" color="#409eff"><scale-to-original /></el-icon>
                <div class="divider-line"></div>
              </div>

              <div class="comparison-col comparison-last">
                <div class="col-header">
                  <el-tag type="info" size="small" effect="light">上次诊断</el-tag>
                  <span v-if="lastDiagnosis" class="last-time">{{ lastDiagnosis.timestamp }}</span>
                </div>
                <div v-if="pair.last" class="col-body" :class="'severity-' + severityClass(pair.last.severity)">
                  <div class="col-description">{{ pair.last.description }}</div>
                  <div class="col-param-row">
                    <span class="col-param-label">关联参数:</span>
                    <span class="col-param-name">{{ pair.last.related_param }}</span>
                  </div>
                  <div class="col-param-compare">
                    <div class="col-param-block">
                      <div class="col-param-label-sm">当时值</div>
                      <div class="col-param-value current-v">{{ formatValue(pair.last.current_value) }}</div>
                    </div>
                    <div class="col-arrow">
                      <el-icon :size="18"><right /></el-icon>
                    </div>
                    <div class="col-param-block">
                      <div class="col-param-label-sm">推荐值</div>
                      <div class="col-param-value recommended-v">{{ formatValue(pair.last.recommended_value) }}</div>
                    </div>
                  </div>
                </div>
                <div v-else class="col-body col-empty">
                  <div class="empty-icon-wrap">
                    <el-icon :size="32" color="#67c23a"><circle-check /></el-icon>
                  </div>
                  <div class="empty-text">上次无此问题</div>
                  <div class="col-param-row placeholder-row">
                    <span class="col-param-label">关联参数:</span>
                    <span class="col-param-name placeholder">—</span>
                  </div>
                  <div class="col-param-compare">
                    <div class="col-param-block placeholder-block">
                      <div class="col-param-label-sm">当时值</div>
                      <div class="col-param-value placeholder-v">—</div>
                    </div>
                    <div class="col-arrow placeholder-arrow">
                      <el-icon :size="18"><right /></el-icon>
                    </div>
                    <div class="col-param-block placeholder-block">
                      <div class="col-param-label-sm">推荐值</div>
                      <div class="col-param-value placeholder-v">—</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="apply-actions">
          <el-button
            type="success"
            size="large"
            @click="applyRecommendations"
            :disabled="applying || props.simulating"
            :loading="applying || props.simulating"
          >
            <el-icon style="margin-right: 6px"><check /></el-icon>
            {{ props.simulating ? '仿真运行中...' : '一键应用推荐参数' }}
          </el-button>
          <el-button size="large" @click="clearDiagnosis" :disabled="props.simulating">
            清除诊断结果
          </el-button>
          <el-button
            size="large"
            type="warning"
            @click="exportReport"
            :disabled="exportingReport"
            :loading="exportingReport"
          >
            <el-icon style="margin-right: 6px"><download /></el-icon>
            {{ exportingReport ? '正在生成报告...' : '导出诊断报告' }}
          </el-button>
        </div>
      </div>

      <div v-else-if="diagnosisResult && diagnosisResult.conclusions.length === 0" class="diagnosis-results">
        <el-result icon="success" title="系统工况正常" sub-title="当前仿真结果未发现明显工况问题，各项指标均在合理范围内">
          <template #extra>
            <el-button type="primary" @click="clearDiagnosis">确定</el-button>
            <el-button type="warning" @click="exportReport">
              <el-icon style="margin-right: 4px"><download /></el-icon>
              导出诊断报告
            </el-button>
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
                  :color="severityColor(getMaxSeverity(record.conclusions))"
                  size="small"
                  effect="dark"
                  style="margin-left: 6px; border: none"
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
                  <el-tag :color="severityColor(item.severity)" size="default" effect="dark" style="border: none">
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
import { ref, computed, watch, reactive, nextTick, onMounted } from 'vue'
import { simulationApi } from '../api'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Search, Check, Delete, Clock, Right,
  FirstAidKit, TrendCharts, Cpu, Sunny, SetUp,
  ScaleToOriginal, Download, CaretBottom, CaretTop,
  Document, Aim, Loading, CircleCheck, ArrowUp, ArrowDown, Minus
} from '@element-plus/icons-vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  MarkLineComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

use([
  LineChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  MarkLineComponent,
  CanvasRenderer
])

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
  },
  simulating: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['apply-params', 'run-simulation'])

const diagnosing = ref(false)
const applying = ref(false)
const exportingReport = ref(false)
const diagnosisResult = ref(null)
const historyList = ref([])
const historyDialogVisible = ref(false)
const activeHistoryId = ref(null)
const activeHistoryRecord = ref(null)

const compareMode = ref(false)
const expandedTimeSeries = reactive({})
const expandedDetails = ref([])

const estimateResult = reactive({})
const estimateResultAbsolute = reactive({})
const estimateLoading = reactive({})

const HISTORY_KEY = 'wastewater_sim_diagnosis_history'

const hasResult = computed(() => !!props.simulationResult)

const lastDiagnosis = computed(() => {
  if (historyList.value.length < 1) return null
  return historyList.value[0]
})

const categoryToIndicator = {
  '硝化': 'nh3_n',
  '反硝化': 'tn',
  '除磷': 'tp',
  '有机物降解': 'cod',
  '固液分离': 'ss'
}

const indicatorLabels = {
  'nh3_n': 'NH3-N',
  'tn': 'TN',
  'tp': 'TP',
  'cod': 'COD',
  'ss': 'SS',
  'bod5': 'BOD5'
}

const paramPathToSensitivityName = {
  'aeration_rate': 'aeration_rate',
  'internal_recirculation_ratio': 'internal_recirculation_ratio',
  'srt': 'srt',
  'anoxic.do_setpoint': 'do_anoxic',
  'anaerobic.do_setpoint': 'do_anaerobic',
  'anaerobic.hrt': 'hrt_anaerobic',
  'clarifier.hrt': 'hrt_clarifier'
}

const severityClass = (severity) => {
  if (severity === '轻度') return 'mild'
  if (severity === '中度') return 'moderate'
  if (severity === '重度') return 'severe'
  return 'mild'
}

const severityColor = (severity) => {
  if (severity === '轻度') return '#d4a017'
  if (severity === '中度') return '#e6770e'
  if (severity === '重度') return '#f56c6c'
  return '#909399'
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

const getIndicatorKey = (category) => categoryToIndicator[category] || 'cod'

const getIndicatorLabel = (category) => indicatorLabels[getIndicatorKey(category)] || category

const getComplianceLimit = (category) => {
  if (!props.simulationResult || !props.simulationResult.compliance) return 0
  const key = getIndicatorKey(category)
  const comp = props.simulationResult.compliance
  if (comp[key] && comp[key][1] !== undefined) return comp[key][1]
  return 0
}

const getCurrentIndicatorValue = (category) => {
  if (!props.simulationResult || !props.simulationResult.final_effluent) return 0
  const key = getIndicatorKey(category)
  return props.simulationResult.final_effluent[key] || 0
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
  if (compareMode.value && historyList.value.length < 1) {
    compareMode.value = false
  }
}

const viewHistory = (record) => {
  activeHistoryId.value = record.id
  activeHistoryRecord.value = record
  historyDialogVisible.value = true
}

const toggleTimeSeries = (index) => {
  expandedTimeSeries[index] = !expandedTimeSeries[index]
}

const buildTimeSeriesOption = (item, index) => {
  if (!props.simulationResult || !props.simulationResult.time_series) return {}
  const ts = props.simulationResult.time_series
  const key = getIndicatorKey(item.category)
  const limit = getComplianceLimit(item.category)

  const data = ts.map(t => {
    const val = t.final_effluent[key]
    return Number(val?.toFixed?.(3) ?? val ?? 0)
  })

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params) => {
        const hour = params[0].axisValue
        const day = (Number(hour) / 24).toFixed(1)
        return `第 ${day} 天 (${hour}h)<br/>${indicatorLabels[key]}: <b>${params[0].value}</b> mg/L`
      }
    },
    grid: { left: 50, right: 20, top: 20, bottom: 30 },
    xAxis: {
      type: 'category',
      data: ts.map(t => t.time_hours),
      axisLabel: {
        formatter: (v) => (v / 24).toFixed(0) + 'd',
        fontSize: 10
      }
    },
    yAxis: {
      type: 'value',
      name: `${indicatorLabels[key]} (mg/L)`,
      nameTextStyle: { fontSize: 10 },
      min: 0,
      axisLabel: { fontSize: 10 }
    },
    series: [{
      name: indicatorLabels[key],
      type: 'line',
      smooth: true,
      showSymbol: false,
      data,
      lineStyle: { width: 2, color: '#409eff' },
      areaStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(64, 158, 255, 0.25)' },
            { offset: 1, color: 'rgba(64, 158, 255, 0.02)' }
          ]
        }
      },
      markLine: {
        silent: true,
        symbol: 'none',
        lineStyle: { color: '#f56c6c', type: 'dashed', width: 1.5 },
        label: {
          formatter: `限值 ${limit}`,
          position: 'end',
          color: '#f56c6c',
          fontSize: 10
        },
        data: [{ yAxis: limit }]
      }
    }]
  }
}

const estimateStatusClass = (index) => {
  if (estimateLoading[index]) return 'loading'
  if (estimateResult[index] === null || estimateResult[index] === undefined) return 'na'
  return estimateResult[index] >= 0 ? 'up' : 'down'
}

const estimateTrendClass = (val) => {
  if (val > 0) return 'trend-up'
  if (val < 0) return 'trend-down'
  return 'trend-flat'
}

const runSensitivityEstimate = async (conclusion, index) => {
  estimateLoading[index] = true
  estimateResult[index] = null
  estimateResultAbsolute[index] = null

  const paramName = conclusion.param_path
  const sensitivityName = paramPathToSensitivityName[paramName]
  const indicatorKey = getIndicatorKey(conclusion.category)

  if (!sensitivityName) {
    estimateLoading[index] = false
    return
  }

  try {
    const currentValue = conclusion.current_value
    const recommendedValue = conclusion.recommended_value
    const delta = recommendedValue - currentValue

    let minVal, maxVal
    if (Math.abs(delta) < 0.0001) {
      estimateLoading[index] = false
      estimateResult[index] = 0
      estimateResultAbsolute[index] = getCurrentIndicatorValue(conclusion.category)
      return
    }

    if (delta > 0) {
      minVal = currentValue
      maxVal = recommendedValue * 1.1
    } else {
      minVal = Math.max(0, recommendedValue * 0.9)
      maxVal = currentValue
    }

    const res = await simulationApi.sensitivity({
      param_name: sensitivityName,
      min_value: minVal,
      max_value: maxVal,
      steps: 5,
      influent: { ...props.influent },
      process_config: JSON.parse(JSON.stringify(props.processConfig)),
      sim_config: { time_step_hours: 1, total_duration_days: 3 }
    })

    if (res.success && res.data && res.data.points && res.data.points.length >= 2) {
      const points = res.data.points
      const currentIndicator = getCurrentIndicatorValue(conclusion.category)

      const idxCurrent = points.reduce((bestIdx, p, i) => {
        const best = points[bestIdx]
        return Math.abs(p.param_value - currentValue) < Math.abs(best.param_value - currentValue) ? i : bestIdx
      }, 0)

      const idxRecommended = points.reduce((bestIdx, p, i) => {
        const best = points[bestIdx]
        return Math.abs(p.param_value - recommendedValue) < Math.abs(best.param_value - recommendedValue) ? i : bestIdx
      }, 0)

      let estimatedIndicator
      if (idxCurrent === idxRecommended) {
        if (points.length >= 2) {
          const i1 = 0, i2 = points.length - 1
          const p1 = points[i1], p2 = points[i2]
          const slope = (p2[indicatorKey] - p1[indicatorKey]) / (p2.param_value - p1.param_value || 1)
          estimatedIndicator = p1[indicatorKey] + slope * (recommendedValue - p1.param_value)
        } else {
          estimatedIndicator = currentIndicator
        }
      } else {
        const pCur = points[idxCurrent]
        const pRec = points[idxRecommended]
        const ratio = (recommendedValue - pCur.param_value) / (pRec.param_value - pCur.param_value || 1)
        estimatedIndicator = pCur[indicatorKey] + ratio * (pRec[indicatorKey] - pCur[indicatorKey])
      }

      const absoluteChange = estimatedIndicator - currentIndicator
      const percentChange = currentIndicator > 0 ? (absoluteChange / currentIndicator) * 100 : 0

      estimateResult[index] = percentChange
      estimateResultAbsolute[index] = Math.max(0, estimatedIndicator)
    }
  } catch (e) {
    console.warn('灵敏度预估失败', e)
  } finally {
    estimateLoading[index] = false
  }
}

const getCompareTrendIcon = (pair) => {
  if (!pair.current || !pair.last) return Minus
  const diff = pair.current.recommended_value - pair.last.recommended_value
  const threshold = Math.abs(pair.last.recommended_value) * 0.001
  if (diff > threshold) return ArrowUp
  if (diff < -threshold) return ArrowDown
  return Minus
}

const getCompareTrendText = (pair) => {
  if (!pair.current || !pair.last) return ''
  const diff = pair.current.recommended_value - pair.last.recommended_value
  const threshold = Math.abs(pair.last.recommended_value) * 0.001
  if (diff > threshold) return '上升'
  if (diff < -threshold) return '下降'
  return '不变'
}

const getCompareTrendClass = (pair) => {
  if (!pair.current || !pair.last) return 'trend-flat'
  const diff = pair.current.recommended_value - pair.last.recommended_value
  const threshold = Math.abs(pair.last.recommended_value) * 0.001
  if (diff > threshold) return 'trend-up'
  if (diff < -threshold) return 'trend-down'
  return 'trend-flat'
}

const mergedComparisonList = computed(() => {
  const current = diagnosisResult.value?.conclusions || []
  const last = lastDiagnosis.value?.conclusions || []

  const pairs = []
  const matchedLast = new Set()

  for (const c of current) {
    const matchKey = c.description
    const lastIdx = last.findIndex((l, i) => !matchedLast.has(i) && l.description === matchKey)
    if (lastIdx >= 0) {
      matchedLast.add(lastIdx)
      pairs.push({ category: c.category, description: c.description, current: c, last: last[lastIdx] })
    } else {
      pairs.push({ category: c.category, description: c.description, current: c, last: null })
    }
  }

  for (let i = 0; i < last.length; i++) {
    if (!matchedLast.has(i)) {
      pairs.push({ category: last[i].category, description: last[i].description, current: null, last: last[i] })
    }
  }

  return pairs
})

const startDiagnosis = async () => {
  if (!props.simulationResult) {
    ElMessage.warning('请先运行一次仿真')
    return
  }

  diagnosing.value = true
  diagnosisResult.value = null
  for (const k of Object.keys(expandedTimeSeries)) delete expandedTimeSeries[k]
  expandedDetails.value = []

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
        nextTick(() => {
          res.data.conclusions.forEach((c, i) => {
            runSensitivityEstimate(c, i)
          })
        })
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
  }).catch(() => {
    applying.value = false
  })
}

const clearDiagnosis = () => {
  diagnosisResult.value = null
  compareMode.value = false
  for (const k of Object.keys(expandedTimeSeries)) delete expandedTimeSeries[k]
  expandedDetails.value = []
}

const renderChartToBase64 = async (item, index) => {
  const option = buildTimeSeriesOption(item, index)
  if (!option || !option.series) return null

  try {
    const echarts = await import('echarts')
    const container = document.createElement('div')
    container.style.width = '600px'
    container.style.height = '200px'
    container.style.position = 'absolute'
    container.style.left = '-9999px'
    document.body.appendChild(container)

    const chart = echarts.init(container)
    chart.setOption(option)

    await new Promise(r => setTimeout(r, 300))

    const base64 = chart.getDataURL({ type: 'png', pixelRatio: 2, backgroundColor: '#fff' })
    chart.dispose()
    document.body.removeChild(container)
    return base64
  } catch (e) {
    console.warn('图表转base64失败', e)
    return null
  }
}

const escapeHtml = (str) => {
  if (str === null || str === undefined) return ''
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

const generateReportSummary = (conclusions) => {
  if (!conclusions || conclusions.length === 0) {
    return '本次诊断未发现工况问题，系统各项指标运行正常，当前工艺配置合理。建议继续保持现有运行参数，定期巡检监测。'
  }

  const severe = conclusions.filter(c => c.severity === '重度').length
  const moderate = conclusions.filter(c => c.severity === '中度').length
  const mild = conclusions.filter(c => c.severity === '轻度').length

  const priorityMap = {}
  for (const c of conclusions) {
    if (!priorityMap[c.category]) priorityMap[c.category] = 0
    priorityMap[c.category] += { '重度': 3, '中度': 2, '轻度': 1 }[c.severity] || 1
  }
  const sortedCats = Object.entries(priorityMap).sort((a, b) => b[1] - a[1])
  const topCategory = sortedCats[0]?.[0] || '参数'

  let summary = `本次诊断共发现 ${conclusions.length} 条工况问题，其中重度 ${severe} 条、中度 ${moderate} 条、轻度 ${mild} 条。`
  summary += `建议优先调整【${topCategory}】相关参数。`

  if (severe > 0) {
    const severeItems = conclusions.filter(c => c.severity === '重度')
    summary += `特别关注重度问题：${severeItems.slice(0, 3).map(c => c.description).join('、')}。`
  }

  summary += '按推荐值调整后可有效改善出水水质，降低超标风险。'
  return summary
}

const exportReport = async () => {
  if (!diagnosisResult.value) {
    ElMessage.warning('请先完成诊断')
    return
  }

  exportingReport.value = true

  try {
    const now = new Date()
    const pad = n => String(n).padStart(2, '0')
    const reportTime = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`
    const conclusions = diagnosisResult.value.conclusions || []

    const chartImages = {}
    for (let i = 0; i < conclusions.length; i++) {
      const img = await renderChartToBase64(conclusions[i], i)
      if (img) chartImages[i] = img
    }

    const influentRows = [
      ['COD', props.influent.cod, 'mg/L'],
      ['BOD5', props.influent.bod5, 'mg/L'],
      ['SS', props.influent.ss, 'mg/L'],
      ['TN', props.influent.tn, 'mg/L'],
      ['NH3-N', props.influent.nh3_n, 'mg/L'],
      ['TP', props.influent.tp, 'mg/L'],
      ['pH', props.influent.ph, ''],
      ['水温', props.influent.temperature, '℃']
    ]

    const processRows = [
      ['日处理量', props.processConfig.daily_flow, 'm³/d'],
      ['曝气量', props.processConfig.aeration_rate, 'm³/h'],
      ['污泥龄SRT', props.processConfig.srt, '天'],
      ['污泥回流比', props.processConfig.sludge_recirculation_ratio, ''],
      ['内回流比', props.processConfig.internal_recirculation_ratio, ''],
      ['厌氧池 HRT', props.processConfig.anaerobic.hrt, 'h'],
      ['厌氧池 MLSS', props.processConfig.anaerobic.mlss, 'mg/L'],
      ['厌氧池 DO', props.processConfig.anaerobic.do_setpoint, 'mg/L'],
      ['缺氧池 HRT', props.processConfig.anoxic.hrt, 'h'],
      ['缺氧池 MLSS', props.processConfig.anoxic.mlss, 'mg/L'],
      ['缺氧池 DO', props.processConfig.anoxic.do_setpoint, 'mg/L'],
      ['好氧池 HRT', props.processConfig.aerobic.hrt, 'h'],
      ['好氧池 MLSS', props.processConfig.aerobic.mlss, 'mg/L'],
      ['好氧池 DO', props.processConfig.aerobic.do_setpoint, 'mg/L'],
      ['二沉池 HRT', props.processConfig.clarifier.hrt, 'h'],
      ['二沉池 MLSS', props.processConfig.clarifier.mlss, 'mg/L']
    ]

    const summary = generateReportSummary(conclusions)

    const severityBg = { '轻度': '#fff8e6', '中度': '#fff3e0', '重度': '#fdecea' }
    const severityFg = { '轻度': '#d4a017', '中度': '#e6770e', '重度': '#f56c6c' }

    let diagnosisHtml = ''
    if (conclusions.length === 0) {
      diagnosisHtml = '<div style="padding:30px; text-align:center; background:#f0f9eb; border-radius:8px; border:1px solid #e1f3d8;"><div style="font-size:48px; color:#67c23a;">✓</div><div style="font-size:18px; font-weight:bold; color:#67c23a; margin-top:10px;">系统工况正常</div><div style="color:#606266; margin-top:6px;">当前仿真结果未发现明显工况问题</div></div>'
    } else {
      diagnosisHtml = conclusions.map((c, i) => {
        const est = estimateResult[i]
        const estHtml = (est !== null && est !== undefined)
          ? `<div style="margin-top:8px; padding:8px 12px; background:#f5f7fa; border-radius:6px; font-size:12px; color:${est >= 0 ? '#f56c6c' : '#67c23a'};">
              <b>预估影响：</b>预计出水${getIndicatorLabel(c.category)} ${est >= 0 ? '上升' : '下降'} ${Math.abs(est).toFixed(1)}%
            </div>`
          : ''

        const chartHtml = chartImages[i]
          ? `<div style="margin-top:12px; padding:12px; background:#fafafa; border-radius:6px; border:1px solid #ebeef5;">
              <div style="font-size:12px; color:#909399; margin-bottom:8px;">出水${getIndicatorLabel(c.category)}时序变化（限值: ${getComplianceLimit(c.category)} mg/L）</div>
              <img src="${chartImages[i]}" style="width:100%; max-width:600px; display:block;" />
            </div>`
          : ''

        return `
          <div style="margin-bottom:16px; padding:16px; border-radius:8px; border:1px solid #ebeef5; border-left:4px solid ${severityFg[c.severity] || '#909399'}; background:${severityBg[c.severity] || '#fff'};">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px;">
              <div style="font-weight:bold; font-size:15px; color:#303133;">
                【${escapeHtml(c.category)}】${escapeHtml(c.description)}
              </div>
              <span style="padding:3px 10px; border-radius:4px; background:${severityFg[c.severity] || '#909399'}; color:#fff; font-size:12px; font-weight:bold;">
                ${escapeHtml(c.severity)}
              </span>
            </div>
            <div style="font-size:13px; color:#606266; margin-bottom:10px;">
              关联参数：<b>${escapeHtml(c.related_param)}</b>
            </div>
            <div style="display:flex; gap:12px; align-items:center;">
              <div style="flex:1; padding:10px; text-align:center; background:#fef0f0; border-radius:6px;">
                <div style="font-size:11px; color:#909399;">当前值</div>
                <div style="font-size:18px; font-weight:bold; color:#f56c6c;">${formatValue(c.current_value)}</div>
              </div>
              <div style="font-size:20px; color:#c0c4cc;">→</div>
              <div style="flex:1; padding:10px; text-align:center; background:#f0f9eb; border-radius:6px;">
                <div style="font-size:11px; color:#909399;">推荐值</div>
                <div style="font-size:18px; font-weight:bold; color:#67c23a;">${formatValue(c.recommended_value)}</div>
              </div>
            </div>
            <div style="margin-top:10px; padding:8px 12px; background:#f5f7fa; border-radius:6px; font-size:12px; color:#606266; line-height:1.6;">
              <b>推荐算法说明：</b>${escapeHtml(c.recommendation_detail)}
            </div>
            ${estHtml}
            ${chartHtml}
          </div>
        `
      }).join('')
    }

    const html = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>污水处理工况诊断报告 - ${reportTime}</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif; background: #f5f7fa; color: #303133; line-height: 1.6; padding: 24px; }
    .report-container { max-width: 900px; margin: 0 auto; background: #fff; border-radius: 12px; box-shadow: 0 4px 20px rgba(0,0,0,0.08); overflow: hidden; }
    .report-header { background: linear-gradient(135deg, #409eff 0%, #66b1ff 100%); color: #fff; padding: 28px 32px; }
    .report-title { font-size: 22px; font-weight: bold; margin-bottom: 6px; display: flex; align-items: center; gap: 10px; }
    .report-sub { font-size: 13px; opacity: 0.9; }
    .report-section { padding: 24px 32px; border-bottom: 1px solid #f0f0f0; }
    .section-title { font-size: 16px; font-weight: bold; margin-bottom: 16px; padding-left: 12px; border-left: 4px solid #409eff; color: #303133; }
    .info-table { width: 100%; border-collapse: collapse; font-size: 13px; }
    .info-table th { background: #f5f7fa; padding: 10px 12px; text-align: left; color: #606266; font-weight: 600; border-bottom: 1px solid #ebeef5; }
    .info-table td { padding: 10px 12px; border-bottom: 1px solid #f5f5f5; color: #303133; }
    .info-table tr:hover td { background: #fafafa; }
    .info-table td.value { font-weight: 600; text-align: right; font-family: 'Consolas', monospace; }
    .two-col-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
    .summary-box { padding: 16px 20px; background: #ecf5ff; border: 1px solid #d9ecff; border-radius: 8px; color: #409eff; font-size: 13px; line-height: 1.8; }
    .summary-box b { color: #303133; }
    .report-footer { padding: 20px 32px; text-align: center; font-size: 12px; color: #909399; background: #fafafa; }
    @media (max-width: 768px) { .two-col-grid { grid-template-columns: 1fr; } }
  </style>
</head>
<body>
  <div class="report-container">
    <div class="report-header">
      <div class="report-title">
        <span style="font-size:26px;">⚙️</span>
        污水处理工艺智能工况诊断报告
      </div>
      <div class="report-sub">
        报告生成时间：${reportTime} &nbsp;|&nbsp;
        A²/O 工艺仿真与诊断系统
      </div>
    </div>

    <div class="report-section">
      <div class="summary-box">
        ${summary}
      </div>
    </div>

    <div class="report-section">
      <div class="section-title">一、进水参数</div>
      <table class="info-table">
        <thead>
          <tr><th style="width:25%;">参数名称</th><th style="width:35%;" class="value">数值</th><th>单位</th></tr>
        </thead>
        <tbody>
          ${influentRows.map(r => `<tr><td>${escapeHtml(r[0])}</td><td class="value">${formatValue(r[1])}</td><td>${escapeHtml(r[2])}</td></tr>`).join('')}
        </tbody>
      </table>
    </div>

    <div class="report-section">
      <div class="section-title">二、当前工艺配置参数</div>
      <div class="two-col-grid">
        <div>
          <div style="font-size:13px; font-weight:600; color:#409eff; margin-bottom:8px;">全局参数</div>
          <table class="info-table">
            <tbody>
              ${processRows.slice(0, 5).map(r => `<tr><td>${escapeHtml(r[0])}</td><td class="value">${formatValue(r[1])}</td><td>${escapeHtml(r[2])}</td></tr>`).join('')}
            </tbody>
          </table>
        </div>
        <div>
          <div style="font-size:13px; font-weight:600; color:#409eff; margin-bottom:8px;">单元池体参数</div>
          <table class="info-table">
            <tbody>
              ${processRows.slice(5).map(r => `<tr><td>${escapeHtml(r[0])}</td><td class="value">${formatValue(r[1])}</td><td>${escapeHtml(r[2])}</td></tr>`).join('')}
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="report-section">
      <div class="section-title">三、诊断结论列表 ${conclusions.length > 0 ? `（共 ${conclusions.length} 条）` : ''}</div>
      ${diagnosisHtml}
    </div>

    <div class="report-footer">
      本报告由污水处理工艺仿真与诊断系统自动生成 · 仅供参考，请结合实际运行情况综合判断
    </div>
  </div>
</body>
</html>`

    const blob = new Blob([html], { type: 'text/html;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    const safeTime = reportTime.replace(/[: ]/g, '-')
    a.href = url
    a.download = `工况诊断报告_${safeTime}.html`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    setTimeout(() => URL.revokeObjectURL(url), 1000)

    ElMessage.success('诊断报告导出成功')
  } catch (e) {
    console.error('导出报告失败', e)
    ElMessage.error('导出报告失败: ' + (e.message || '未知错误'))
  } finally {
    exportingReport.value = false
  }
}

watch(() => props.simulating, (newVal, oldVal) => {
  if (oldVal && !newVal) {
    applying.value = false
  }
})

watch(compareMode, (val) => {
  if (val && historyList.value.length < 1) {
    ElMessage.warning('暂无历史诊断记录，无法开启对比模式')
    compareMode.value = false
  }
})

onMounted(() => {
  loadHistory()
})
</script>

<style scoped>
.diagnosis-panel {
  width: 100%;
}

.diagnosis-toolbar {
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.switch-hint {
  font-size: 12px;
  color: #909399;
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
  border-left: 4px solid #d4a017;
}

.diagnosis-card.severity-moderate {
  border-left: 4px solid #e6770e;
  background: #fff8f0;
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
  gap: 12px;
}

.card-category {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 4px;
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

.card-timeseries {
  margin-top: 12px;
  padding: 14px 16px;
  background: #fafafa;
  border: 1px dashed #dcdfe6;
  border-radius: 8px;
}

.timeseries-title {
  display: flex;
  align-items: center;
  font-size: 13px;
  font-weight: 600;
  color: #606266;
  margin-bottom: 10px;
  gap: 6px;
}

.timeseries-chart {
  width: 100%;
  height: 150px;
}

.card-detail {
  margin-top: 4px;
}

.collapse-title {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: #409eff;
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

.estimate-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 13px;
}

.estimate-label {
  color: #909399;
  font-weight: 600;
  white-space: nowrap;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.estimate-value {
  flex: 1;
  color: #606266;
  line-height: 1.7;
}

.estimate-value.loading {
  color: #909399;
}

.estimate-value.na {
  color: #c0c4cc;
  font-style: italic;
}

.estimate-value.up .trend-up {
  color: #f56c6c;
}

.estimate-value.down .trend-down {
  color: #67c23a;
}

.estimate-loading {
  display: inline-flex;
  align-items: center;
  gap: 4px;
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
  flex-wrap: wrap;
}

/* 对比模式样式 */
.diagnosis-cards.compare-mode {
  gap: 20px;
}

.comparison-card-wrapper {
  border: 1px solid #ebeef5;
  border-radius: 10px;
  overflow: hidden;
  background: #fff;
  transition: all 0.3s;
}

.comparison-card-wrapper:hover {
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}

.comparison-card-titlebar {
  padding: 14px 20px;
  background: linear-gradient(90deg, #ecf5ff 0%, #f5f7fa 100%);
  border-bottom: 1px solid #ebeef5;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.comparison-title-label {
  font-weight: 600;
  font-size: 14px;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 6px;
}

.comparison-title-badges {
  display: flex;
  align-items: center;
}

.comparison-columns {
  display: flex;
  gap: 0;
  min-height: 220px;
}

.comparison-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.col-header {
  padding: 10px 16px;
  background: #fafafa;
  border-bottom: 1px solid #f0f0f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.last-time {
  font-size: 11px;
  color: #909399;
}

.col-body {
  flex: 1;
  padding: 16px;
  border-radius: 0;
}

.col-body.severity-mild {
  background: #fffef5;
}

.col-body.severity-moderate {
  background: #fff8f0;
}

.col-body.severity-severe {
  background: #fef0f0;
}

.col-description {
  font-size: 13px;
  color: #606266;
  margin-bottom: 10px;
  line-height: 1.6;
}

.col-param-row {
  font-size: 12px;
  margin-bottom: 10px;
}

.col-param-label {
  color: #909399;
  margin-right: 4px;
}

.col-param-name {
  color: #303133;
  font-weight: 600;
}

.col-param-compare {
  display: flex;
  align-items: center;
  gap: 8px;
}

.col-param-block {
  flex: 1;
  text-align: center;
  padding: 8px 10px;
  border-radius: 6px;
}

.col-param-label-sm {
  font-size: 11px;
  color: #909399;
  margin-bottom: 3px;
}

.col-param-value {
  font-size: 16px;
  font-weight: 700;
}

.col-param-value.current-v {
  color: #f56c6c;
  background: #fef0f0;
  border-radius: 4px;
  padding: 4px;
}

.col-param-value.recommended-v {
  color: #67c23a;
  background: #f0f9eb;
  border-radius: 4px;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex-wrap: wrap;
}

.col-arrow {
  color: #c0c4cc;
  flex-shrink: 0;
}

.trend-arrow {
  font-size: 11px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.7);
}

.trend-arrow.trend-up {
  color: #f56c6c;
}

.trend-arrow.trend-down {
  color: #67c23a;
}

.trend-arrow.trend-flat {
  color: #909399;
}

.comparison-divider {
  width: 48px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: #fafafa;
  border-left: 1px solid #f0f0f0;
  border-right: 1px solid #f0f0f0;
}

.divider-line {
  flex: 1;
  width: 1px;
  background: #e4e7ed;
  min-height: 20px;
}

.col-body.col-empty {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 10px;
  color: #909399;
  background: #fafafa;
}

.empty-icon-wrap {
  display: flex;
  justify-content: center;
  margin-top: 4px;
}

.empty-text {
  text-align: center;
  font-size: 13px;
  font-weight: 500;
  color: #67c23a;
  margin-bottom: 4px;
}

.placeholder-row .placeholder {
  color: #c0c4cc;
  font-weight: 500;
}

.placeholder-block {
  opacity: 0.6;
}

.placeholder-arrow {
  color: #e4e7ed;
}

.placeholder-v {
  background: #f5f7fa !important;
  color: #c0c4cc !important;
  border: 1px dashed #e4e7ed;
  font-weight: 500 !important;
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

