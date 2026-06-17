<template>
  <div class="app-container">
    <div class="app-header">
      <h1>
        <el-icon style="margin-right: 12px; vertical-align: -3px"><setting /></el-icon>
        污水处理工艺仿真与出水水质预测工具
      </h1>
      <p>基于ASM1简化模型的A²/O工艺仿真系统 | 支持参数灵敏度分析与能耗估算</p>
    </div>

    <div class="main-layout">
      <div class="left-panel">
        <div class="panel" style="margin-bottom: 24px">
          <div class="panel-header">
            <h3><el-icon><setting /></el-icon>参数配置</h3>
          </div>
          <div class="panel-body">
            <InfluentInput v-model="influent" />
            <ProcessConfig v-model="processConfig" />

            <div class="form-section">
              <div class="form-section-title">仿真参数</div>
              <el-form :model="simConfig" label-width="120px" size="default" label-position="left">
                <el-form-item label="时间步长">
                  <el-input-number v-model="simConfig.time_step_hours" :min="0.5" :max="4" :step="0.5" :controls="false" style="width: 100%" />
                  <template #append>h</template>
                </el-form-item>
                <el-form-item label="仿真时长">
                  <el-input-number v-model="simConfig.total_duration_days" :min="1" :max="30" :step="1" :controls="false" style="width: 100%" />
                  <template #append>天</template>
                </el-form-item>
              </el-form>
            </div>

            <div class="form-section">
              <div class="form-section-title">方案管理</div>
              <div class="plan-save-bar">
                <el-input
                  v-model="newPlanName"
                  placeholder="输入方案名称后保存"
                  size="default"
                  style="flex: 1; margin-right: 8px"
                  maxlength="20"
                  show-word-limit
                />
                <el-button
                  type="primary"
                  :disabled="!newPlanName.trim() || savedPlans.length >= 5"
                  @click="savePlan"
                >
                  <el-icon><folder-add /></el-icon>
                  <span style="margin-left: 4px">保存</span>
                </el-button>
              </div>
              <el-alert
                v-if="savedPlans.length >= 5"
                type="warning"
                :closable="false"
                show-icon
                style="margin-top: 8px; font-size: 12px"
              >
                最多保存5个方案，请先删除不需要的方案
              </el-alert>

              <div class="plans-list" v-if="savedPlans.length > 0">
                <div
                  v-for="(plan, index) in savedPlans"
                  :key="plan.id"
                  class="plan-item"
                >
                  <div class="plan-info">
                    <div class="plan-name">
                      <el-tag size="small" :type="tagColors[index % tagColors.length]" effect="light">方案{{ index + 1 }}</el-tag>
                      <span class="plan-title">{{ plan.name }}</span>
                    </div>
                    <div class="plan-meta">{{ plan.createdAt }}</div>
                  </div>
                  <div class="plan-actions">
                    <el-button size="small" type="primary" link @click="loadPlan(plan)">
                      <el-icon><refresh-left /></el-icon>
                      <span style="margin-left: 2px">加载</span>
                    </el-button>
                    <el-button size="small" type="danger" link @click="deletePlan(plan.id)">
                      <el-icon><delete /></el-icon>
                      <span style="margin-left: 2px">删除</span>
                    </el-button>
                  </div>
                </div>
              </div>
              <el-empty v-else description="暂无保存的方案" :image-size="60" style="padding: 16px 0" />
            </div>

            <el-progress
              v-if="simulating || isAnimating || comparing"
              :percentage="progressPercent"
              :status="simulating || comparing ? '' : 'success'"
              style="margin-bottom: 16px"
            >
              <template #default="{ percentage }">
                <span style="font-size: 13px">
                  {{ comparing ? '对比仿真中...' : simulating ? '计算中...' : '动画演示中...' }} {{ percentage }}%
                </span>
              </template>
            </el-progress>

            <el-button
              type="primary"
              size="large"
              style="width: 100%"
              class="btn-primary"
              :loading="simulating || comparing"
              @click="runSimulation"
            >
              <el-icon style="margin-right: 6px"><video-play /></el-icon>
              {{ simulating ? '仿真计算中...' : comparing ? '对比仿真中...' : '开始仿真' }}
            </el-button>
          </div>
        </div>
      </div>

      <div class="right-panel">
        <el-tabs v-model="activeResultTab" type="card">
          <el-tab-pane label="仿真结果" name="results">
            <div class="panel">
              <div class="panel-header">
                <h3><el-icon><data-analysis /></el-icon>仿真结果</h3>
              </div>
              <div class="panel-body">
                <SimulationResults
                  :result="simulationResult"
                  :animate="true"
                  :process-config="processConfig"
                  :influent="influent"
                  :sim-config="simConfig"
                  @animation-done="onAnimationDone"
                  @progress="updateAnimationProgress"
                />
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="方案对比" name="compare">
            <div class="panel">
              <div class="panel-header">
                <h3><el-icon><scale-to-original /></el-icon>方案对比</h3>
              </div>
              <div class="panel-body">
                <div v-if="savedPlans.length < 2" class="empty-state">
                  <div class="icon"><el-icon :size="56"><warning /></el-icon></div>
                  <p>请先保存至少2个方案后再进行对比（最多勾选3个方案）</p>
                  <div style="margin-top: 16px">
                    <el-tag type="info">当前已保存方案数: {{ savedPlans.length }}</el-tag>
                  </div>
                </div>

                <div v-else>
                  <div class="compare-select-section">
                    <div class="compare-select-title">
                      选择要对比的方案
                      <el-tag type="info" size="small" style="margin-left: 8px">已选 {{ selectedPlans.length }}/3</el-tag>
                    </div>
                    <el-checkbox-group v-model="selectedPlans" :max="3" class="compare-checkbox-group">
                      <el-checkbox
                        v-for="(plan, index) in savedPlans"
                        :key="plan.id"
                        :label="plan.id"
                        border
                      >
                        <el-tag size="small" :type="tagColors[index % tagColors.length]" effect="light">方案{{ index + 1 }}</el-tag>
                        <span style="margin-left: 6px">{{ plan.name }}</span>
                      </el-checkbox>
                    </el-checkbox-group>
                    <div class="compare-actions">
                      <el-button
                        type="primary"
                        :disabled="selectedPlans.length < 2 || comparing"
                        :loading="comparing"
                        @click="runCompareSimulation"
                        size="default"
                      >
                        <el-icon style="margin-right: 4px"><video-play /></el-icon>
                        对比仿真
                      </el-button>
                      <el-button
                        size="default"
                        @click="selectedPlans = []"
                        :disabled="comparing"
                      >
                        清空选择
                      </el-button>
                    </div>
                  </div>

                  <div v-if="compareResults.length > 0">
                    <div class="result-section">
                      <div class="result-section-title">
                        <el-icon><trend-charts /></el-icon>
                        出水NH3-N变化对比
                      </div>
                      <v-chart class="chart-container" :option="nh3nCompareOption" autoresize />
                    </div>

                    <div class="result-section">
                      <div class="result-section-title">
                        <el-icon><trend-charts /></el-icon>
                        出水TN变化对比
                      </div>
                      <v-chart class="chart-container" :option="tnCompareOption" autoresize />
                    </div>

                    <div class="result-section">
                      <div class="result-section-title">
                        <el-icon><grid /></el-icon>
                        最终指标与能耗对比
                      </div>
                      <el-table :data="compareTableData" size="small" border stripe>
                        <el-table-column prop="planName" label="方案" width="140" fixed>
                          <template #default="{ row, $index }">
                            <el-tag :type="tagColors[$index % tagColors.length]">{{ row.planName }}</el-tag>
                          </template>
                        </el-table-column>
                        <el-table-column prop="cod" label="COD (mg/L)" width="110" align="right" />
                        <el-table-column prop="bod5" label="BOD5 (mg/L)" width="110" align="right" />
                        <el-table-column prop="ss" label="SS (mg/L)" width="100" align="right" />
                        <el-table-column prop="nh3_n" label="NH3-N (mg/L)" width="110" align="right" />
                        <el-table-column prop="tn" label="TN (mg/L)" width="100" align="right" />
                        <el-table-column prop="tp" label="TP (mg/L)" width="100" align="right" />
                        <el-table-column prop="kwh" label="吨水电耗 (kWh/m³)" width="140" align="right" />
                        <el-table-column prop="compliant" label="达标" width="90" align="center">
                          <template #default="{ row }">
                            <el-tag :type="row.compliant ? 'success' : 'danger'" size="small">
                              {{ row.compliant ? '达标' : '超标' }}
                            </el-tag>
                          </template>
                        </el-table-column>
                      </el-table>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="参数灵敏度分析" name="sensitivity">
            <div class="panel">
              <div class="panel-header">
                <h3><el-icon><aim /></el-icon>参数灵敏度分析</h3>
              </div>
              <div class="panel-body">
                <SensitivityAnalysis
                  :influent="influent"
                  :process-config="processConfig"
                  :sim-config="simConfig"
                />
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="工况诊断" name="diagnosis">
            <div class="panel">
              <div class="panel-header">
                <h3><el-icon><first-aid-kit /></el-icon>智能工况诊断与参数优化推荐</h3>
              </div>
              <div class="panel-body">
                <DiagnosisPanel
                  :simulation-result="simulationResult"
                  :influent="influent"
                  :process-config="processConfig"
                  @apply-params="onApplyDiagnosisParams"
                />
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { Setting, DataAnalysis, Aim, VideoPlay, FolderAdd, RefreshLeft, Delete, ScaleToOriginal, Warning, FirstAidKit } from '@element-plus/icons-vue'
import InfluentInput from './components/InfluentInput.vue'
import ProcessConfig from './components/ProcessConfig.vue'
import SimulationResults from './components/SimulationResults.vue'
import SensitivityAnalysis from './components/SensitivityAnalysis.vue'
import DiagnosisPanel from './components/DiagnosisPanel.vue'
import { simulationApi } from './api'
import { ElMessage, ElMessageBox } from 'element-plus'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

use([
  LineChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  CanvasRenderer
])

const STORAGE_KEY = 'wastewater_sim_saved_plans'

const activeResultTab = ref('results')
const simulating = ref(false)
const comparing = ref(false)
const isAnimating = ref(false)
const simulationResult = ref(null)
const computeProgress = ref(0)
const animationProgress = ref(0)
const compareProgress = ref(0)
let computeProgressTimer = null

const newPlanName = ref('')
const savedPlans = ref([])
const selectedPlans = ref([])
const compareResults = ref([])

const tagColors = ['primary', 'success', 'warning', 'danger', 'info']
const lineColors = ['#667eea', '#f56c6c', '#67c23a', '#e6a23c', '#909399']

const progressPercent = computed(() => {
  if (comparing.value) {
    return compareProgress.value
  }
  if (simulating.value) {
    return computeProgress.value
  }
  if (isAnimating.value) {
    return 70 + Math.floor(animationProgress.value * 0.3)
  }
  return 0
})

const influent = reactive({
  cod: 300,
  bod5: 150,
  ss: 200,
  tn: 40,
  nh3_n: 25,
  tp: 5,
  ph: 7.2,
  temperature: 20
})

const processConfig = reactive({
  anaerobic: { volume: 4000, hrt: 1.92, mlss: 4000, do_setpoint: 0.1 },
  anoxic: { volume: 6000, hrt: 2.88, mlss: 4000, do_setpoint: 0.3 },
  aerobic: { volume: 15000, hrt: 7.2, mlss: 4000, do_setpoint: 3.0 },
  clarifier: { volume: 7500, hrt: 3.6, mlss: 8000, do_setpoint: 0.0 },
  sludge_recirculation_ratio: 1.0,
  internal_recirculation_ratio: 2.5,
  srt: 18,
  aeration_rate: 3000,
  daily_flow: 50000
})

const simConfig = reactive({
  time_step_hours: 1,
  total_duration_days: 7
})

const loadSavedPlans = () => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      savedPlans.value = JSON.parse(stored)
    }
  } catch (e) {
    console.warn('加载方案失败', e)
    savedPlans.value = []
  }
}

const persistPlans = () => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(savedPlans.value))
  } catch (e) {
    console.warn('保存方案失败', e)
  }
}

onMounted(async () => {
  loadSavedPlans()
  try {
    const res = await simulationApi.getTemplates()
    if (res.success && res.data && res.data.medium) {
      Object.assign(processConfig, JSON.parse(JSON.stringify(res.data.medium)))
    }
  } catch (e) {
    console.warn('模板加载失败，使用默认值', e)
  }
})

const generateId = () => Date.now().toString(36) + Math.random().toString(36).substr(2, 5)

const formatDate = (d) => {
  const pad = n => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

const savePlan = () => {
  const name = newPlanName.value.trim()
  if (!name) {
    ElMessage.warning('请输入方案名称')
    return
  }
  if (savedPlans.value.length >= 5) {
    ElMessage.warning('最多保存5个方案')
    return
  }

  const plan = {
    id: generateId(),
    name,
    createdAt: formatDate(new Date()),
    influent: JSON.parse(JSON.stringify(influent)),
    processConfig: JSON.parse(JSON.stringify(processConfig)),
    simConfig: JSON.parse(JSON.stringify(simConfig))
  }
  savedPlans.value.push(plan)
  persistPlans()
  newPlanName.value = ''
  ElMessage.success(`方案"${name}"已保存`)
}

const loadPlan = (plan) => {
  Object.assign(influent, JSON.parse(JSON.stringify(plan.influent)))
  Object.assign(processConfig, JSON.parse(JSON.stringify(plan.processConfig)))
  Object.assign(simConfig, JSON.parse(JSON.stringify(plan.simConfig)))
  ElMessage.success(`已加载方案"${plan.name}"`)
  activeResultTab.value = 'results'
}

const deletePlan = (id) => {
  ElMessageBox.confirm('确定要删除该方案吗？', '提示', {
    type: 'warning',
    confirmButtonText: '确定',
    cancelButtonText: '取消'
  }).then(() => {
    const idx = savedPlans.value.findIndex(p => p.id === id)
    if (idx >= 0) {
      const name = savedPlans.value[idx].name
      savedPlans.value.splice(idx, 1)
      persistPlans()
      selectedPlans.value = selectedPlans.value.filter(sid => sid !== id)
      ElMessage.success(`已删除方案"${name}"`)
    }
  }).catch(() => {})
}

const runSimulation = async () => {
  simulating.value = true
  comparing.value = false
  isAnimating.value = false
  computeProgress.value = 0
  animationProgress.value = 0
  simulationResult.value = null
  activeResultTab.value = 'results'

  const totalSteps = Math.ceil(simConfig.total_duration_days * 24 / simConfig.time_step_hours)
  const progressPerTick = 70 / Math.max(totalSteps / 5, 10)
  computeProgressTimer = setInterval(() => {
    computeProgress.value = Math.min(69, computeProgress.value + progressPerTick)
  }, 100)

  try {
    const res = await simulationApi.simulate({
      influent: { ...influent },
      process_config: JSON.parse(JSON.stringify(processConfig)),
      sim_config: { ...simConfig }
    })

    if (computeProgressTimer) {
      clearInterval(computeProgressTimer)
      computeProgressTimer = null
    }
    computeProgress.value = 70

    if (res.data) {
      simulationResult.value = res.data
      isAnimating.value = true
      if (res.data.success) {
        ElMessage.success('仿真完成，正在展示结果...')
      } else {
        ElMessage.warning(res.data.message)
      }
    } else {
      simulating.value = false
      ElMessage.error(res.message || '仿真失败')
    }
  } catch (e) {
    if (computeProgressTimer) {
      clearInterval(computeProgressTimer)
      computeProgressTimer = null
    }
    simulating.value = false
    ElMessage.error('仿真失败: ' + (e.message || '未知错误'))
  }
}

const onAnimationDone = () => {
  simulating.value = false
  isAnimating.value = false
  animationProgress.value = 100
}

const updateAnimationProgress = (percent) => {
  animationProgress.value = percent
}

const getPlanById = (id) => savedPlans.value.find(p => p.id === id)

const runCompareSimulation = async () => {
  if (selectedPlans.value.length < 2) {
    ElMessage.warning('请至少选择2个方案')
    return
  }

  comparing.value = true
  simulating.value = false
  compareProgress.value = 0
  compareResults.value = []
  activeResultTab.value = 'compare'

  const planIds = [...selectedPlans.value]
  const total = planIds.length
  const results = []

  try {
    for (let i = 0; i < planIds.length; i++) {
      const plan = getPlanById(planIds[i])
      if (!plan) continue

      compareProgress.value = Math.floor((i / total) * 90)

      const res = await simulationApi.simulate({
        influent: { ...plan.influent },
        process_config: JSON.parse(JSON.stringify(plan.processConfig)),
        sim_config: { ...plan.simConfig }
      })

      if (res.data && res.data.success) {
        results.push({
          plan,
          result: res.data
        })
      } else {
        ElMessage.warning(`方案"${plan.name}"仿真失败: ${res.data?.message || '未知错误'}`)
      }
    }

    compareProgress.value = 100
    compareResults.value = results

    if (results.length >= 2) {
      ElMessage.success(`对比仿真完成，成功${results.length}个方案`)
    } else {
      ElMessage.warning(`对比完成，但仅${results.length}个方案成功`)
    }
  } catch (e) {
    ElMessage.error('对比仿真出错: ' + (e.message || '未知错误'))
  } finally {
    comparing.value = false
  }
}

const buildCompareChartOption = (indicator, indicatorLabel) => {
  if (compareResults.value.length === 0) return {}

  const series = compareResults.value.map((item, idx) => {
    const plan = item.plan
    const ts = item.result.time_series
    const planIdx = savedPlans.value.findIndex(p => p.id === plan.id)
    return {
      name: `${planIdx + 1}. ${plan.name}`,
      type: 'line',
      smooth: true,
      showSymbol: false,
      data: ts.map(t => Number(indicator(t).toFixed(2))),
      itemStyle: { color: lineColors[planIdx % lineColors.length] },
      lineStyle: { width: 2.5 }
    }
  })

  const firstTs = compareResults.value[0].result.time_series

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params) => {
        let html = `第 ${(Number(params[0].axisValue) / 24).toFixed(1)} 天<br/>`
        params.forEach(p => {
          html += `${p.marker} ${p.seriesName}: <b>${p.value}</b> mg/L<br/>`
        })
        return html
      }
    },
    legend: {
      data: series.map(s => s.name),
      top: 0
    },
    grid: { left: 60, right: 30, top: 50, bottom: 50 },
    xAxis: {
      type: 'category',
      data: firstTs.map(t => t.time_hours),
      name: '时间 (h)',
      nameLocation: 'middle',
      nameGap: 30,
      axisLabel: {
        formatter: (v) => (v / 24).toFixed(0) + 'd'
      }
    },
    yAxis: {
      type: 'value',
      name: `${indicatorLabel} (mg/L)`,
      min: 0
    },
    series
  }
}

const nh3nCompareOption = computed(() =>
  buildCompareChartOption(
    (t) => t.final_effluent.nh3_n,
    'NH3-N'
  )
)

const tnCompareOption = computed(() =>
  buildCompareChartOption(
    (t) => t.final_effluent.tn,
    'TN'
  )
)

const compareTableData = computed(() => {
  return compareResults.value.map(item => {
    const plan = item.plan
    const c = item.result.compliance
    const planIdx = savedPlans.value.findIndex(p => p.id === plan.id)
    return {
      planName: `方案${planIdx + 1} - ${plan.name}`,
      cod: c.cod[0].toFixed(2),
      bod5: c.bod5[0].toFixed(2),
      ss: c.ss[0].toFixed(2),
      nh3_n: c.nh3_n[0].toFixed(2),
      tn: c.tn[0].toFixed(2),
      tp: c.tp[0].toFixed(3),
      kwh: item.result.energy_consumption.kwh_per_m3.toFixed(3),
      compliant: c.overall_compliant
    }
  })
})

watch(activeResultTab, (tab) => {
  if (tab === 'compare' && compareResults.value.length > 0) {
    setTimeout(() => {
      window.dispatchEvent(new Event('resize'))
    }, 100)
  }
})

const onApplyDiagnosisParams = (updatedConfig) => {
  Object.assign(processConfig, updatedConfig)
  activeResultTab.value = 'results'
  nextTick(() => {
    runSimulation()
  })
}

onBeforeUnmount(() => {
  if (computeProgressTimer) {
    clearInterval(computeProgressTimer)
    computeProgressTimer = null
  }
})
</script>

<style scoped>
.left-panel {
  position: sticky;
  top: 20px;
}

.plan-save-bar {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.plans-list {
  max-height: 240px;
  overflow-y: auto;
  padding-right: 4px;
}

.plan-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: #fafafa;
  border-radius: 8px;
  margin-bottom: 8px;
  border: 1px solid #ebeef5;
  transition: all 0.2s;
}

.plan-item:hover {
  background: #f5f7fa;
  border-color: #dcdfe6;
}

.plan-info {
  flex: 1;
  min-width: 0;
}

.plan-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 2px;
}

.plan-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.plan-meta {
  font-size: 11px;
  color: #909399;
}

.plan-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.compare-select-section {
  background: #fafafa;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 24px;
  border: 1px solid #ebeef5;
}

.compare-select-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 12px;
}

.compare-checkbox-group {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 16px;
}

.compare-actions {
  display: flex;
  gap: 10px;
}

.chart-container {
  width: 100%;
  height: 340px;
}

.result-section {
  margin-bottom: 28px;
}

.result-section-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
