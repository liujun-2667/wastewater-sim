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
        <v-chart class="chart-container" :option="nitrogenChartOption" autoresize />
      </div>

      <div class="result-section">
        <div class="result-section-title">
          <el-icon><trend-charts /></el-icon>
          主要出水指标变化
        </div>
        <v-chart class="chart-container" :option="mainChartOption" autoresize />
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
          <v-chart class="chart-container" :option="energyPieOption" autoresize />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { SankeyChart, PieChart } from 'echarts/charts'

use([SankeyChart, PieChart])

const props = defineProps({
  result: {
    type: Object,
    default: null
  }
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
  if (!props.result || !props.result.time_series.length) return {}
  const ts = props.result.time_series
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
      name: '浓度 (mg/L)'
    },
    series: [
      {
        name: 'NH3-N',
        type: 'line',
        smooth: true,
        data: ts.map(t => t.aerobic_effluent.nh3_n.toFixed(2)),
        itemStyle: { color: '#f56c6c' }
      },
      {
        name: 'NO3-N',
        type: 'line',
        smooth: true,
        data: ts.map(t => t.aerobic_effluent.no3_n.toFixed(2)),
        itemStyle: { color: '#409eff' }
      },
      {
        name: 'TN',
        type: 'line',
        smooth: true,
        data: ts.map(t => t.aerobic_effluent.tn.toFixed(2)),
        itemStyle: { color: '#67c23a' }
      }
    ]
  }
})

const mainChartOption = computed(() => {
  if (!props.result || !props.result.time_series.length) return {}
  const ts = props.result.time_series
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
      name: '浓度 (mg/L)'
    },
    series: [
      {
        name: 'COD',
        type: 'line',
        smooth: true,
        data: ts.map(t => t.final_effluent.cod.toFixed(1)),
        itemStyle: { color: '#667eea' }
      },
      {
        name: 'BOD5',
        type: 'line',
        smooth: true,
        data: ts.map(t => t.final_effluent.bod5.toFixed(1)),
        itemStyle: { color: '#764ba2' }
      },
      {
        name: 'SS',
        type: 'line',
        smooth: true,
        data: ts.map(t => t.final_effluent.ss.toFixed(1)),
        itemStyle: { color: '#e6a23c' }
      },
      {
        name: 'TP',
        type: 'line',
        smooth: true,
        data: ts.map(t => t.final_effluent.tp.toFixed(3)),
        itemStyle: { color: '#909399' }
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
</style>
