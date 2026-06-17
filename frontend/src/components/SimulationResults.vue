<template>
  <div class="simulation-results">
    <div v-if="!result" class="empty-state">
      <div class="icon">
        <el-icon :size="64"><data-analysis /></el-icon>
      </div>
      <p>请先配置参数并点击"开始仿真"按钮</p>
    </div>

    <div v-else>
      <div class="result-section">
        <div class="result-section-header">
          <div class="result-section-title">
            <el-icon><data-line /></el-icon>
            工艺流程可视化
          </div>
          <el-button type="primary" size="default" @click="exportReport">
            <el-icon style="margin-right: 4px"><download /></el-icon>
            导出报告
          </el-button>
        </div>
        <ProcessFlowDiagram
          :time-series-step="currentTimeStep"
          :unit-effluents="result.unit_effluents"
          :process-config="processConfig"
        />
        <div class="time-indicator" v-if="visibleTimeSeries.length > 0">
          <span>当前时刻: 第 {{ (currentTimeStep?.time_hours / 24).toFixed(1) || 0 }} 天</span>
        </div>
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><circle-check /></el-icon>
          仿真状态
        </div>
        <el-alert
          :type="result.success ? 'success' : 'error'"
          :closable="false"
          show-icon
          style="margin-bottom: 12px"
        >
          {{ result.message }}
        </el-alert>
        <el-alert
          v-if="result.influent_warnings && result.influent_warnings.length > 0"
          type="warning"
          :closable="false"
          show-icon
          style="margin-bottom: 12px"
        >
          <div v-for="(w, i) in result.influent_warnings" :key="i" style="font-size: 13px">{{ w }}</div>
        </el-alert>
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><trophy /></el-icon>
          达标判定 (一级A标)
        </div>
        <el-table :data="complianceRows" size="small" border>
          <el-table-column prop="name" label="指标" width="100" />
          <el-table-column prop="value" label="出水值" width="120">
            <template #default="{ row }">
              <span :class="row.compliant ? 'compliant' : 'non-compliant'">
                {{ row.value }}
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="limit" label="标准限值" width="120" />
          <el-table-column prop="compliant" label="达标" width="80">
            <template #default="{ row }">
              <el-tag :type="row.compliant ? 'success' : 'danger'" size="small">
                {{ row.compliant ? '达标' : '超标' }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
        <div style="margin-top: 12px">
          <el-tag :type="result.compliance.overall_compliant ? 'success' : 'danger'" size="large">
            {{ result.compliance.overall_compliant ? '✓ 总体达标' : '✗ 未达标' }}
          </el-tag>
        </div>
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><grid /></el-icon>
          各单元出水水质
        </div>
        <el-table :data="qualityRows" size="small" border>
          <el-table-column prop="indicator" label="指标" width="90" fixed />
          <el-table-column prop="anaerobic" label="厌氧池" width="95" align="right" />
          <el-table-column prop="anoxic" label="缺氧池" width="95" align="right" />
          <el-table-column prop="aerobic" label="好氧池" width="95" align="right" />
          <el-table-column prop="clarifier" label="二沉池(出水)" width="120" align="right">
            <template #default="{ row }">
              <b>{{ row.clarifier }}</b>
            </template>
          </el-table-column>
          <el-table-column prop="unit" label="单位" width="70" />
        </el-table>
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><trend-charts /></el-icon>
          好氧池出水氮形态变化
        </div>
        <v-chart ref="nitrogenChartRef" class="chart-container" :option="nitrogenChartOption" autoresize />
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><trend-charts /></el-icon>
          主要出水指标变化
        </div>
        <v-chart ref="mainChartRef" class="chart-container" :option="mainChartOption" autoresize />
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><share /></el-icon>
          氮物料平衡桑基图
        </div>
        <v-chart class="sankey-container" :option="sankeyOption(result.mass_balance.nitrogen_flows)" autoresize />
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><share /></el-icon>
          磷物料平衡桑基图
        </div>
        <v-chart class="sankey-container" :option="sankeyOption(result.mass_balance.phosphorus_flows)" autoresize />
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><lightning /></el-icon>
          能耗估算
        </div>
        <el-row :gutter="16">
          <el-col :span="12">
            <div class="stat-card">
              <div class="label">总能耗 (kWh/日)</div>
              <div class="value">{{ result.energy_consumption.total_kwh.toFixed(1) }}</div>
            </div>
          </el-col>
          <el-col :span="12">
            <div class="stat-card">
              <div class="label">吨水电耗 (kWh/m³)</div>
              <div class="value">{{ result.energy_consumption.kwh_per_m3.toFixed(3) }}</div>
            </div>
          </el-col>
        </el-row>
        <div style="margin-top: 16px">
          <v-chart ref="energyChartRef" class="chart-container" :option="energyPieOption" autoresize />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart, BarChart, SankeyChart, PieChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import ProcessFlowDiagram from './ProcessFlowDiagram.vue'
import { DataLine, Download } from '@element-plus/icons-vue'
import * as echarts from 'echarts/core'

use([
  LineChart,
  BarChart,
  SankeyChart,
  PieChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  CanvasRenderer
])

const props = defineProps({
  result: {
    type: Object,
    default: null
  },
  animate: {
    type: Boolean,
    default: true
  },
  processConfig: {
    type: Object,
    required: true
  },
  influent: {
    type: Object,
    required: true
  },
  simConfig: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['animation-done', 'progress'])

const displayStepCount = ref(0)
let animationTimer = null
const nitrogenChartRef = ref(null)
const mainChartRef = ref(null)
const energyChartRef = ref(null)

watch(
  () => props.result,
  (newVal) => {
    if (animationTimer) {
      clearInterval(animationTimer)
      animationTimer = null
    }
    if (!newVal || !newVal.time_series || !newVal.time_series.length) {
      displayStepCount.value = 0
      return
    }
    if (!props.animate) {
      displayStepCount.value = newVal.time_series.length
      emit('animation-done')
      emit('progress', 100)
      return
    }
    displayStepCount.value = 0
    const total = newVal.time_series.length
    const stepsPerTick = Math.max(1, Math.ceil(total / 80))
    const interval = Math.max(30, 1000 / 40)
    animationTimer = setInterval(() => {
      displayStepCount.value += stepsPerTick
      const percent = Math.min(100, Math.floor(displayStepCount.value / total * 100))
      emit('progress', percent)
      if (displayStepCount.value >= total) {
        displayStepCount.value = total
        clearInterval(animationTimer)
        animationTimer = null
        emit('animation-done')
        emit('progress', 100)
      }
    }, interval)
  },
  { immediate: true }
)

onBeforeUnmount(() => {
  if (animationTimer) {
    clearInterval(animationTimer)
    animationTimer = null
  }
})

const visibleTimeSeries = computed(() => {
  if (!props.result || !props.result.time_series) return []
  return props.result.time_series.slice(0, displayStepCount.value)
})

const currentTimeStep = computed(() => {
  if (visibleTimeSeries.value.length === 0) return null
  return visibleTimeSeries.value[visibleTimeSeries.value.length - 1]
})

const complianceRows = computed(() => {
  if (!props.result) return []
  const c = props.result.compliance
  return [
    { name: 'COD', value: c.cod[0].toFixed(2), limit: `≤${c.cod[1]}`, compliant: c.cod[2] },
    { name: 'BOD5', value: c.bod5[0].toFixed(2), limit: `≤${c.bod5[1]}`, compliant: c.bod5[2] },
    { name: 'SS', value: c.ss[0].toFixed(2), limit: `≤${c.ss[1]}`, compliant: c.ss[2] },
    { name: 'TN', value: c.tn[0].toFixed(2), limit: `≤${c.tn[1]}`, compliant: c.tn[2] },
    { name: 'NH3-N', value: c.nh3_n[0].toFixed(2), limit: `≤${c.nh3_n[1]}`, compliant: c.nh3_n[2] },
    { name: 'TP', value: c.tp[0].toFixed(3), limit: `≤${c.tp[1]}`, compliant: c.tp[2] }
  ]
})

const qualityRows = computed(() => {
  if (!props.result) return []
  const u = props.result.unit_effluents
  return [
    { indicator: 'COD', anaerobic: u.anaerobic.cod.toFixed(1), anoxic: u.anoxic.cod.toFixed(1), aerobic: u.aerobic.cod.toFixed(1), clarifier: u.clarifier.cod.toFixed(1), unit: 'mg/L' },
    { indicator: 'BOD5', anaerobic: u.anaerobic.bod5.toFixed(1), anoxic: u.anoxic.bod5.toFixed(1), aerobic: u.aerobic.bod5.toFixed(1), clarifier: u.clarifier.bod5.toFixed(1), unit: 'mg/L' },
    { indicator: 'SS', anaerobic: u.anaerobic.ss.toFixed(1), anoxic: u.anoxic.ss.toFixed(1), aerobic: u.aerobic.ss.toFixed(1), clarifier: u.clarifier.ss.toFixed(1), unit: 'mg/L' },
    { indicator: 'TN', anaerobic: u.anaerobic.tn.toFixed(1), anoxic: u.anoxic.tn.toFixed(1), aerobic: u.aerobic.tn.toFixed(1), clarifier: u.clarifier.tn.toFixed(1), unit: 'mg/L' },
    { indicator: 'NH3-N', anaerobic: u.anaerobic.nh3_n.toFixed(2), anoxic: u.anoxic.nh3_n.toFixed(2), aerobic: u.aerobic.nh3_n.toFixed(2), clarifier: u.clarifier.nh3_n.toFixed(2), unit: 'mg/L' },
    { indicator: 'NO3-N', anaerobic: u.anaerobic.no3_n.toFixed(2), anoxic: u.anoxic.no3_n.toFixed(2), aerobic: u.aerobic.no3_n.toFixed(2), clarifier: u.clarifier.no3_n.toFixed(2), unit: 'mg/L' },
    { indicator: 'TP', anaerobic: u.anaerobic.tp.toFixed(2), anoxic: u.anoxic.tp.toFixed(2), aerobic: u.aerobic.tp.toFixed(2), clarifier: u.clarifier.tp.toFixed(3), unit: 'mg/L' }
  ]
})

const nitrogenChartOption = computed(() => {
  if (!visibleTimeSeries.value.length) return {}
  const ts = visibleTimeSeries.value
  return {
    tooltip: { trigger: 'axis' },
    legend: { data: ['NH3-N', 'NO3-N', 'TN'], top: 0 },
    grid: { left: 50, right: 30, top: 40, bottom: 40 },
    xAxis: {
      type: 'category',
      data: ts.map(t => (t.time_hours / 24).toFixed(1)),
      name: '时间 (天)',
      nameLocation: 'middle',
      nameGap: 28
    },
    yAxis: {
      type: 'value',
      name: '浓度 (mg/L)',
      min: 0
    },
    series: [
      {
        name: 'NH3-N',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: ts.map(t => Number(t.aerobic_effluent.nh3_n.toFixed(2))),
        itemStyle: { color: '#f56c6c' },
        lineStyle: { width: 2 }
      },
      {
        name: 'NO3-N',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: ts.map(t => Number(t.aerobic_effluent.no3_n.toFixed(2))),
        itemStyle: { color: '#409eff' },
        lineStyle: { width: 2 }
      },
      {
        name: 'TN',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: ts.map(t => Number(t.aerobic_effluent.tn.toFixed(2))),
        itemStyle: { color: '#67c23a' },
        lineStyle: { width: 2 }
      }
    ]
  }
})

const mainChartOption = computed(() => {
  if (!visibleTimeSeries.value.length) return {}
  const ts = visibleTimeSeries.value
  return {
    tooltip: { trigger: 'axis' },
    legend: { data: ['COD', 'BOD5', 'SS', 'TP'], top: 0 },
    grid: { left: 50, right: 30, top: 40, bottom: 40 },
    xAxis: {
      type: 'category',
      data: ts.map(t => (t.time_hours / 24).toFixed(1)),
      name: '时间 (天)',
      nameLocation: 'middle',
      nameGap: 28
    },
    yAxis: {
      type: 'value',
      name: '浓度 (mg/L)',
      min: 0
    },
    series: [
      {
        name: 'COD',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: ts.map(t => Number(t.final_effluent.cod.toFixed(1))),
        itemStyle: { color: '#667eea' },
        lineStyle: { width: 2 }
      },
      {
        name: 'BOD5',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: ts.map(t => Number(t.final_effluent.bod5.toFixed(1))),
        itemStyle: { color: '#764ba2' },
        lineStyle: { width: 2 }
      },
      {
        name: 'SS',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: ts.map(t => Number(t.final_effluent.ss.toFixed(1))),
        itemStyle: { color: '#e6a23c' },
        lineStyle: { width: 2 }
      },
      {
        name: 'TP',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: ts.map(t => Number(t.final_effluent.tp.toFixed(3))),
        itemStyle: { color: '#909399' },
        lineStyle: { width: 2 }
      }
    ]
  }
})

const sankeyOption = (flows) => {
  const nodeMap = new Map()
  flows.forEach(f => {
    nodeMap.set(f[0], true)
    nodeMap.set(f[1], true)
  })
  const nodes = Array.from(nodeMap.keys()).map(name => ({ name }))
  const links = flows.map(f => ({
    source: f[0],
    target: f[1],
    value: Number(f[2].toFixed(3))
  }))

  return {
    tooltip: {
      trigger: 'item',
      formatter: (params) => {
        if (params.dataType === 'node') {
          return params.name
        }
        return `${params.data.source} → ${params.data.target}<br/>${params.data.value.toFixed(2)} kg/日`
      }
    },
    series: [{
      type: 'sankey',
      layout: 'none',
      emphasis: { focus: 'adjacency' },
      data: nodes,
      links: links,
      lineStyle: {
        color: 'gradient',
        curveness: 0.5
      },
      label: {
        fontSize: 11
      }
    }]
  }
}

const energyPieOption = computed(() => {
  if (!props.result) return {}
  const data = props.result.energy_consumption.breakdown.map(b => ({
    name: b[0],
    value: Number(b[1].toFixed(2))
  }))
  return {
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c} kWh ({d}%)'
    },
    legend: { bottom: 0 },
    series: [{
      type: 'pie',
      radius: ['40%', '70%'],
      avoidLabelOverlap: false,
      label: {
        show: true,
        formatter: '{b}\n{d}%'
      },
      data: data,
      color: ['#667eea', '#67c23a', '#e6a23c']
    }]
  }
})

const generateConclusion = () => {
  const c = props.result.compliance
  const overLimit = []
  if (!c.cod[2]) overLimit.push(`COD (${c.cod[0].toFixed(2)} > ${c.cod[1]})`)
  if (!c.bod5[2]) overLimit.push(`BOD5 (${c.bod5[0].toFixed(2)} > ${c.bod5[1]})`)
  if (!c.ss[2]) overLimit.push(`SS (${c.ss[0].toFixed(2)} > ${c.ss[1]})`)
  if (!c.tn[2]) overLimit.push(`TN (${c.tn[0].toFixed(2)} > ${c.tn[1]})`)
  if (!c.nh3_n[2]) overLimit.push(`NH3-N (${c.nh3_n[0].toFixed(2)} > ${c.nh3_n[1]})`)
  if (!c.tp[2]) overLimit.push(`TP (${c.tp[0].toFixed(3)} > ${c.tp[1]})`)

  if (c.overall_compliant) {
    return `本次仿真结果表明，在当前进水水质和工艺参数配置下，A²/O污水处理工艺的各项出水指标均达到国家一级A排放标准。最终出水COD为${c.cod[0].toFixed(2)}mg/L、氨氮为${c.nh3_n[0].toFixed(2)}mg/L、总氮为${c.tn[0].toFixed(2)}mg/L、总磷为${c.tp[0].toFixed(3)}mg/L。系统吨水电耗为${props.result.energy_consumption.kwh_per_m3.toFixed(3)}kWh/m³，能耗水平合理。整体工况运行稳定，处理效果良好。`
  } else {
    return `本次仿真结果表明，在当前进水水质和工艺参数配置下，A²/O污水处理工艺存在以下超标项：${overLimit.join('、')}。总体未达到国家一级A排放标准。建议针对性调整工艺参数，如${!c.nh3_n[2] || !c.tn[2] ? '提高内回流比、延长好氧池HRT以强化硝化反硝化效果' : ''}${!c.tp[2] ? '、优化厌氧池运行条件促进聚磷菌释磷' : ''}${!c.cod[2] || !c.bod5[2] ? '、调整污泥龄SRT保证有机物充分降解' : ''}。当前系统吨水电耗为${props.result.energy_consumption.kwh_per_m3.toFixed(3)}kWh/m³。`
  }
}

const renderChartToDataUrl = (option, width = 800, height = 400) => {
  if (!option || !option.series || !option.series.length) return ''
  const div = document.createElement('div')
  div.style.width = width + 'px'
  div.style.height = height + 'px'
  div.style.position = 'absolute'
  div.style.left = '-9999px'
  div.style.top = '-9999px'
  document.body.appendChild(div)
  try {
    const chart = echarts.init(div)
    chart.setOption(option)
    const dataUrl = chart.getDataURL({ type: 'png', pixelRatio: 2, backgroundColor: '#fff' })
    chart.dispose()
    document.body.removeChild(div)
    return dataUrl
  } catch (e) {
    console.warn('图表渲染失败', e)
    if (div.parentNode) document.body.removeChild(div)
    return ''
  }
}

const buildFullNitrogenOption = (ts) => {
  if (!ts || !ts.length) return {}
  return {
    tooltip: { trigger: 'axis' },
    legend: { data: ['NH3-N', 'NO3-N', 'TN'], top: 0 },
    grid: { left: 50, right: 30, top: 40, bottom: 40 },
    xAxis: {
      type: 'category',
      data: ts.map(t => (t.time_hours / 24).toFixed(1)),
      name: '时间 (天)',
      nameLocation: 'middle',
      nameGap: 28
    },
    yAxis: { type: 'value', name: '浓度 (mg/L)', min: 0 },
    series: [
      { name: 'NH3-N', type: 'line', smooth: true, showSymbol: false, data: ts.map(t => Number(t.aerobic_effluent.nh3_n.toFixed(2))), itemStyle: { color: '#f56c6c' }, lineStyle: { width: 2 } },
      { name: 'NO3-N', type: 'line', smooth: true, showSymbol: false, data: ts.map(t => Number(t.aerobic_effluent.no3_n.toFixed(2))), itemStyle: { color: '#409eff' }, lineStyle: { width: 2 } },
      { name: 'TN', type: 'line', smooth: true, showSymbol: false, data: ts.map(t => Number(t.aerobic_effluent.tn.toFixed(2))), itemStyle: { color: '#67c23a' }, lineStyle: { width: 2 } }
    ]
  }
}

const buildFullMainOption = (ts) => {
  if (!ts || !ts.length) return {}
  return {
    tooltip: { trigger: 'axis' },
    legend: { data: ['COD', 'BOD5', 'SS', 'TP'], top: 0 },
    grid: { left: 50, right: 30, top: 40, bottom: 40 },
    xAxis: {
      type: 'category',
      data: ts.map(t => (t.time_hours / 24).toFixed(1)),
      name: '时间 (天)',
      nameLocation: 'middle',
      nameGap: 28
    },
    yAxis: { type: 'value', name: '浓度 (mg/L)', min: 0 },
    series: [
      { name: 'COD', type: 'line', smooth: true, showSymbol: false, data: ts.map(t => Number(t.final_effluent.cod.toFixed(1))), itemStyle: { color: '#667eea' }, lineStyle: { width: 2 } },
      { name: 'BOD5', type: 'line', smooth: true, showSymbol: false, data: ts.map(t => Number(t.final_effluent.bod5.toFixed(1))), itemStyle: { color: '#764ba2' }, lineStyle: { width: 2 } },
      { name: 'SS', type: 'line', smooth: true, showSymbol: false, data: ts.map(t => Number(t.final_effluent.ss.toFixed(1))), itemStyle: { color: '#e6a23c' }, lineStyle: { width: 2 } },
      { name: 'TP', type: 'line', smooth: true, showSymbol: false, data: ts.map(t => Number(t.final_effluent.tp.toFixed(3))), itemStyle: { color: '#909399' }, lineStyle: { width: 2 } }
    ]
  }
}

const buildFullEnergyPieOption = (result) => {
  if (!result || !result.energy_consumption) return {}
  const data = result.energy_consumption.breakdown.map(b => ({
    name: b[0],
    value: Number(b[1].toFixed(2))
  }))
  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c} kWh ({d}%)' },
    legend: { bottom: 0 },
    series: [{
      type: 'pie',
      radius: ['40%', '70%'],
      avoidLabelOverlap: false,
      label: { show: true, formatter: '{b}\n{d}%' },
      data: data,
      color: ['#667eea', '#67c23a', '#e6a23c']
    }]
  }
}

const exportReport = () => {
  const result = props.result
  if (!result) return

  const inf = { ...props.influent }
  const pc = JSON.parse(JSON.stringify(props.processConfig))
  const sc = { ...props.simConfig }
  const c = result.compliance ? { ...result.compliance } : null
  const ec = result.energy_consumption ? { ...result.energy_consumption } : null
  const ts = result.time_series || []

  const nitrogenChart = renderChartToDataUrl(buildFullNitrogenOption(ts))
  const mainChart = renderChartToDataUrl(buildFullMainOption(ts))
  const energyChart = renderChartToDataUrl(buildFullEnergyPieOption(result))

  const now = new Date()
  const pad2 = n => String(n).padStart(2, '0')
  const timeStr = `${now.getFullYear()}-${pad2(now.getMonth() + 1)}-${pad2(now.getDate())} ${pad2(now.getHours())}:${pad2(now.getMinutes())}`

  const influentRows = [
    ['COD', inf.cod, 'mg/L'],
    ['BOD5', inf.bod5, 'mg/L'],
    ['SS', inf.ss, 'mg/L'],
    ['TN', inf.tn, 'mg/L'],
    ['NH3-N', inf.nh3_n, 'mg/L'],
    ['TP', inf.tp, 'mg/L'],
    ['pH', inf.ph, ''],
    ['水温', inf.temperature, '℃']
  ]

  const processRows = [
    ['日处理量', pc.daily_flow, 'm³/日'],
    ['污泥回流比 R', pc.sludge_recirculation_ratio, ''],
    ['内回流比 r', pc.internal_recirculation_ratio, ''],
    ['污泥龄 SRT', pc.srt, '天'],
    ['曝气量', pc.aeration_rate, 'm³/h'],
    ['厌氧池容积', pc.anaerobic.volume, 'm³'],
    ['厌氧池 HRT', pc.anaerobic.hrt, 'h'],
    ['缺氧池容积', pc.anoxic.volume, 'm³'],
    ['缺氧池 HRT', pc.anoxic.hrt, 'h'],
    ['好氧池容积', pc.aerobic.volume, 'm³'],
    ['好氧池 HRT', pc.aerobic.hrt, 'h'],
    ['二沉池容积', pc.clarifier.volume, 'm³'],
    ['仿真时长', sc.total_duration_days, '天']
  ]

  let complianceHtml = ''
  if (c) {
    const rows = [
      { name: 'COD', value: c.cod[0].toFixed(2), limit: `≤${c.cod[1]}`, compliant: c.cod[2] },
      { name: 'BOD5', value: c.bod5[0].toFixed(2), limit: `≤${c.bod5[1]}`, compliant: c.bod5[2] },
      { name: 'SS', value: c.ss[0].toFixed(2), limit: `≤${c.ss[1]}`, compliant: c.ss[2] },
      { name: 'TN', value: c.tn[0].toFixed(2), limit: `≤${c.tn[1]}`, compliant: c.tn[2] },
      { name: 'NH3-N', value: c.nh3_n[0].toFixed(2), limit: `≤${c.nh3_n[1]}`, compliant: c.nh3_n[2] },
      { name: 'TP', value: c.tp[0].toFixed(3), limit: `≤${c.tp[1]}`, compliant: c.tp[2] }
    ]
    complianceHtml = rows.map(r => `
      <tr>
        <td>${r.name}</td>
        <td class="${r.compliant ? 'compliant' : 'non-compliant'}">${r.value}</td>
        <td>${r.limit}</td>
        <td><span class="tag ${r.compliant ? 'tag-success' : 'tag-danger'}">${r.compliant ? '达标' : '超标'}</span></td>
      </tr>
    `).join('')
  }

  let conclusionText = ''
  if (c) {
    const overLimit = []
    if (!c.cod[2]) overLimit.push(`COD (${c.cod[0].toFixed(2)} > ${c.cod[1]})`)
    if (!c.bod5[2]) overLimit.push(`BOD5 (${c.bod5[0].toFixed(2)} > ${c.bod5[1]})`)
    if (!c.ss[2]) overLimit.push(`SS (${c.ss[0].toFixed(2)} > ${c.ss[1]})`)
    if (!c.tn[2]) overLimit.push(`TN (${c.tn[0].toFixed(2)} > ${c.tn[1]})`)
    if (!c.nh3_n[2]) overLimit.push(`NH3-N (${c.nh3_n[0].toFixed(2)} > ${c.nh3_n[1]})`)
    if (!c.tp[2]) overLimit.push(`TP (${c.tp[0].toFixed(3)} > ${c.tp[1]})`)

    if (c.overall_compliant) {
      conclusionText = `本次仿真结果表明，在当前进水水质和工艺参数配置下，A²/O污水处理工艺的各项出水指标均达到国家一级A排放标准。最终出水COD为${c.cod[0].toFixed(2)}mg/L、氨氮为${c.nh3_n[0].toFixed(2)}mg/L、总氮为${c.tn[0].toFixed(2)}mg/L、总磷为${c.tp[0].toFixed(3)}mg/L。系统吨水电耗为${ec.kwh_per_m3.toFixed(3)}kWh/m³，能耗水平合理。整体工况运行稳定，处理效果良好。`
    } else {
      conclusionText = `本次仿真结果表明，在当前进水水质和工艺参数配置下，A²/O污水处理工艺存在以下超标项：${overLimit.join('、')}。总体未达到国家一级A排放标准。建议针对性调整工艺参数，如${!c.nh3_n[2] || !c.tn[2] ? '提高内回流比、延长好氧池HRT以强化硝化反硝化效果' : ''}${!c.tp[2] ? '、优化厌氧池运行条件促进聚磷菌释磷' : ''}${!c.cod[2] || !c.bod5[2] ? '、调整污泥龄SRT保证有机物充分降解' : ''}。当前系统吨水电耗为${ec.kwh_per_m3.toFixed(3)}kWh/m³。`
    }
  }

  const overallTag = c
    ? `<span class="tag ${c.overall_compliant ? 'tag-success' : 'tag-danger'}" style="font-size: 15px; padding: 6px 16px;">${c.overall_compliant ? '✓ 总体达标' : '✗ 未达标'}</span>`
    : ''

  const html = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>污水处理仿真报告 - ${timeStr}</title>
<style>
  * { box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    max-width: 1000px;
    margin: 0 auto;
    padding: 40px 30px;
    background: #f5f7fa;
    color: #303133;
  }
  .header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 32px;
    border-radius: 12px;
    margin-bottom: 28px;
  }
  .header h1 { margin: 0 0 8px 0; font-size: 26px; }
  .header p { margin: 0; opacity: 0.9; font-size: 14px; }
  .section {
    background: white;
    border-radius: 10px;
    padding: 24px;
    margin-bottom: 20px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.06);
  }
  .section-title {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 16px;
    padding-bottom: 10px;
    border-bottom: 2px solid #667eea;
    color: #303133;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 14px;
  }
  table th, table td {
    border: 1px solid #ebeef5;
    padding: 10px 14px;
    text-align: left;
  }
  table th {
    background: #f5f7fa;
    font-weight: 600;
    color: #606266;
  }
  .compliant { color: #67c23a; font-weight: 600; }
  .non-compliant { color: #f56c6c; font-weight: 600; }
  .tag {
    display: inline-block;
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 13px;
    font-weight: 600;
  }
  .tag-success { background: #f0f9eb; color: #67c23a; border: 1px solid #e1f3d8; }
  .tag-danger { background: #fef0f0; color: #f56c6c; border: 1px solid #fde2e2; }
  .chart-box {
    text-align: center;
    padding: 16px 0;
  }
  .chart-box img {
    max-width: 100%;
    border: 1px solid #ebeef5;
    border-radius: 6px;
  }
  .conclusion {
    line-height: 1.8;
    font-size: 15px;
    color: #606266;
    padding: 16px;
    background: #fafafa;
    border-radius: 8px;
    border-left: 4px solid #667eea;
  }
  .footer {
    text-align: center;
    color: #909399;
    font-size: 12px;
    margin-top: 32px;
    padding-top: 16px;
    border-top: 1px solid #ebeef5;
  }
  .stat-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .stat-card {
    background: #f5f7fa;
    padding: 16px;
    border-radius: 8px;
  }
  .stat-label { font-size: 12px; color: #909399; margin-bottom: 4px; }
  .stat-value { font-size: 22px; font-weight: 700; color: #303133; }
</style>
</head>
<body>

<div class="header">
  <h1>污水处理工艺仿真报告</h1>
  <p>基于ASM1简化模型的A²/O工艺仿真系统 | 报告生成时间: ${timeStr}</p>
</div>

<div class="section">
  <div class="section-title">进水参数</div>
  <table>
    <thead>
      <tr><th>指标</th><th>数值</th><th>单位</th></tr>
    </thead>
    <tbody>
      ${influentRows.map(r => `<tr><td>${r[0]}</td><td><b>${r[1]}</b></td><td>${r[2]}</td></tr>`).join('')}
    </tbody>
  </table>
</div>

<div class="section">
  <div class="section-title">工艺配置参数</div>
  <table>
    <thead>
      <tr><th>参数项</th><th>数值</th><th>单位</th></tr>
    </thead>
    <tbody>
      ${processRows.map(r => `<tr><td>${r[0]}</td><td><b>${r[1]}</b></td><td>${r[2]}</td></tr>`).join('')}
    </tbody>
  </table>
</div>

<div class="section">
  <div class="section-title">达标判定结果 (一级A标)</div>
  <table>
    <thead>
      <tr><th>指标</th><th>出水值</th><th>标准限值</th><th>达标</th></tr>
    </thead>
    <tbody>
      ${complianceHtml}
    </tbody>
  </table>
  <div style="margin-top: 16px;">
    ${overallTag}
  </div>
</div>

<div class="section">
  <div class="section-title">好氧池出水氮形态变化趋势</div>
  <div class="chart-box">
    ${nitrogenChart ? `<img src="${nitrogenChart}" alt="氮形态变化" />` : '<p>图表加载失败</p>'}
  </div>
</div>

<div class="section">
  <div class="section-title">主要出水指标变化趋势</div>
  <div class="chart-box">
    ${mainChart ? `<img src="${mainChart}" alt="主要指标变化" />` : '<p>图表加载失败</p>'}
  </div>
</div>

<div class="section">
  <div class="section-title">能耗分析</div>
  <div class="stat-grid">
    <div class="stat-card">
      <div class="stat-label">总能耗</div>
      <div class="stat-value">${ec ? ec.total_kwh.toFixed(1) : '-'} <span style="font-size: 14px;">kWh/日</span></div>
    </div>
    <div class="stat-card">
      <div class="stat-label">吨水电耗</div>
      <div class="stat-value">${ec ? ec.kwh_per_m3.toFixed(3) : '-'} <span style="font-size: 14px;">kWh/m³</span></div>
    </div>
  </div>
  <div class="chart-box">
    ${energyChart ? `<img src="${energyChart}" alt="能耗饼图" />` : '<p>图表加载失败</p>'}
  </div>
</div>

<div class="section">
  <div class="section-title">结论</div>
  <div class="conclusion">${conclusionText || '暂无结论数据'}</div>
</div>

<div class="footer">
  <p>本报告由污水处理工艺仿真系统自动生成 | 仅供参考，不作为法定依据</p>
</div>

</body>
</html>`

  const blob = new Blob([html], { type: 'text/html;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `污水处理仿真报告_${now.getFullYear()}${pad2(now.getMonth() + 1)}${pad2(now.getDate())}_${pad2(now.getHours())}${pad2(now.getMinutes())}.html`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}
</script>

<style scoped>
.simulation-results {
  width: 100%;
}
.chart-container {
  width: 100%;
  height: 320px;
}
.sankey-container {
  width: 100%;
  height: 380px;
}
.result-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.result-section-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}
.time-indicator {
  margin-top: 12px;
  text-align: center;
  padding: 8px 16px;
  background: linear-gradient(135deg, #667eea20 0%, #764ba220 100%);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  color: #667eea;
}
</style>
