<template>
  <div class="sensitivity-analysis">
    <div class="result-section">
      <div class="result-section-title">
        <el-icon><aim /></el-icon>
        参数灵敏度分析
      </div>

      <el-tabs v-model="activeTab" class="tabs-wrapper">
        <el-tab-pane label="单参数扫描" name="1d">
          <el-form :inline="true" size="small" label-position="left" style="margin-bottom: 16px">
            <el-form-item label="参数">
              <el-select v-model="param1d" style="width: 180px">
                <el-option label="曝气量" value="aeration_rate" />
                <el-option label="污泥回流比" value="sludge_recirculation_ratio" />
                <el-option label="内回流比" value="internal_recirculation_ratio" />
                <el-option label="污泥龄 (SRT)" value="srt" />
                <el-option label="好氧池MLSS" value="mlss_aerobic" />
                <el-option label="好氧池DO" value="do_aerobic" />
              </el-select>
            </el-form-item>
            <el-form-item label="最小值">
              <el-input-number v-model="min1d" :controls="false" style="width: 120px" />
            </el-form-item>
            <el-form-item label="最大值">
              <el-input-number v-model="max1d" :controls="false" style="width: 120px" />
            </el-form-item>
            <el-form-item label="步数">
              <el-input-number v-model="steps1d" :min="3" :max="30" :controls="false" style="width: 100px" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" :loading="loading1d" @click="runAnalysis1d">
                开始分析
              </el-button>
            </el-form-item>
          </el-form>

          <div v-if="result1d">
            <v-chart class="chart-container" :option="sensitivity1dOption" autoresize />
            <el-table :data="result1d.points" size="small" border style="margin-top: 16px">
              <el-table-column :label="param1dLabels[result1d.param_name] || result1d.param_name" prop="param_value" width="120" align="right" />
              <el-table-column label="COD" prop="cod" width="80" align="right">
                <template #default="{ row }">{{ row.cod.toFixed(1) }}</template>
              </el-table-column>
              <el-table-column label="BOD5" prop="bod5" width="80" align="right">
                <template #default="{ row }">{{ row.bod5.toFixed(1) }}</template>
              </el-table-column>
              <el-table-column label="TN" prop="tn" width="80" align="right">
                <template #default="{ row }">{{ row.tn.toFixed(1) }}</template>
              </el-table-column>
              <el-table-column label="NH3-N" prop="nh3_n" width="80" align="right">
                <template #default="{ row }">{{ row.nh3_n.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column label="TP" prop="tp" width="80" align="right">
                <template #default="{ row }">{{ row.tp.toFixed(3) }}</template>
              </el-table-column>
              <el-table-column label="kWh/m³" prop="kwh_per_m3" width="100" align="right">
                <template #default="{ row }">{{ row.kwh_per_m3.toFixed(4) }}</template>
              </el-table-column>
              <el-table-column label="达标" prop="compliant" width="80" align="center">
                <template #default="{ row }">
                  <el-tag :type="row.compliant ? 'success' : 'danger'" size="small">
                    {{ row.compliant ? '达标' : '超标' }}
                  </el-tag>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </el-tab-pane>

        <el-tab-pane label="双参数扫描" name="2d">
          <el-form :inline="true" size="small" label-position="left" style="margin-bottom: 16px">
            <el-form-item label="X参数">
              <el-select v-model="paramX" style="width: 160px">
                <el-option label="曝气量" value="aeration_rate" />
                <el-option label="污泥回流比" value="sludge_recirculation_ratio" />
                <el-option label="内回流比" value="internal_recirculation_ratio" />
                <el-option label="污泥龄 (SRT)" value="srt" />
              </el-select>
            </el-form-item>
            <el-form-item label="X范围">
              <el-input-number v-model="xMin" :controls="false" style="width: 100px" />
              ~
              <el-input-number v-model="xMax" :controls="false" style="width: 100px" />
            </el-form-item>
            <el-form-item label="步数">
              <el-input-number v-model="xSteps" :min="3" :max="15" :controls="false" style="width: 80px" />
            </el-form-item>
          </el-form>
          <el-form :inline="true" size="small" label-position="left" style="margin-bottom: 16px">
            <el-form-item label="Y参数">
              <el-select v-model="paramY" style="width: 160px">
                <el-option label="曝气量" value="aeration_rate" />
                <el-option label="污泥回流比" value="sludge_recirculation_ratio" />
                <el-option label="内回流比" value="internal_recirculation_ratio" />
                <el-option label="污泥龄 (SRT)" value="srt" />
              </el-select>
            </el-form-item>
            <el-form-item label="Y范围">
              <el-input-number v-model="yMin" :controls="false" style="width: 100px" />
              ~
              <el-input-number v-model="yMax" :controls="false" style="width: 100px" />
            </el-form-item>
            <el-form-item label="步数">
              <el-input-number v-model="ySteps" :min="3" :max="15" :controls="false" style="width: 80px" />
            </el-form-item>
          </el-form>
          <el-form :inline="true" size="small" label-position="left" style="margin-bottom: 16px">
            <el-form-item label="目标指标">
              <el-select v-model="targetParam" style="width: 140px">
                <el-option label="COD" value="cod" />
                <el-option label="BOD5" value="bod5" />
                <el-option label="TN" value="tn" />
                <el-option label="NH3-N" value="nh3_n" />
                <el-option label="TP" value="tp" />
                <el-option label="kWh/m³" value="kwh_per_m3" />
              </el-select>
            </el-form-item>
            <el-form-item>
              <el-button type="primary" :loading="loading2d" @click="runAnalysis2d">
                开始分析
              </el-button>
            </el-form-item>
          </el-form>

          <div v-if="result2d">
            <v-chart class="chart-container" :option="sensitivity2dOption" autoresize />
            <div style="text-align: center; color: #909399; font-size: 12px; margin-top: 8px">
              等高线图：颜色越深表示{{ targetParamLabels[result2d.target_param] }}值越高
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart, HeatmapChart, ScatterChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  VisualMapComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import { simulationApi } from '../api'
import { ElMessage } from 'element-plus'

use([
  LineChart,
  HeatmapChart,
  ScatterChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  VisualMapComponent,
  CanvasRenderer
])

const props = defineProps({
  influent: Object,
  processConfig: Object,
  simConfig: Object
})

const activeTab = ref('1d')
const loading1d = ref(false)
const loading2d = ref(false)
const result1d = ref(null)
const result2d = ref(null)

const param1dLabels = {
  aeration_rate: '曝气量 (m³/h)',
  sludge_recirculation_ratio: '污泥回流比',
  internal_recirculation_ratio: '内回流比',
  srt: '污泥龄 (天)',
  mlss_aerobic: '好氧池MLSS (mg/L)',
  do_aerobic: '好氧池DO (mg/L)'
}

const targetParamLabels = {
  cod: 'COD',
  bod5: 'BOD5',
  tn: 'TN',
  nh3_n: 'NH3-N',
  tp: 'TP',
  kwh_per_m3: 'kWh/m³'
}

const param1d = ref('aeration_rate')
const min1d = ref(300)
const max1d = ref(1500)
const steps1d = ref(10)

const paramX = ref('aeration_rate')
const paramY = ref('srt')
const xMin = ref(300)
const xMax = ref(1500)
const yMin = ref(5)
const yMax = ref(30)
const xSteps = ref(8)
const ySteps = ref(8)
const targetParam = ref('tn')

watch([param1d], () => {
  if (param1d.value === 'aeration_rate') { min1d.value = 300; max1d.value = 1500 }
  if (param1d.value === 'sludge_recirculation_ratio') { min1d.value = 0.5; max1d.value = 1.5 }
  if (param1d.value === 'internal_recirculation_ratio') { min1d.value = 1; max1d.value = 4 }
  if (param1d.value === 'srt') { min1d.value = 5; max1d.value = 30 }
  if (param1d.value === 'mlss_aerobic') { min1d.value = 2000; max1d.value = 6000 }
  if (param1d.value === 'do_aerobic') { min1d.value = 1; max1d.value = 5 }
})

watch([paramX], () => {
  if (paramX.value === 'aeration_rate') { xMin.value = 300; xMax.value = 1500 }
  if (paramX.value === 'sludge_recirculation_ratio') { xMin.value = 0.5; xMax.value = 1.5 }
  if (paramX.value === 'internal_recirculation_ratio') { xMin.value = 1; xMax.value = 4 }
  if (paramX.value === 'srt') { xMin.value = 5; xMax.value = 30 }
})

watch([paramY], () => {
  if (paramY.value === 'aeration_rate') { yMin.value = 300; yMax.value = 1500 }
  if (paramY.value === 'sludge_recirculation_ratio') { yMin.value = 0.5; yMax.value = 1.5 }
  if (paramY.value === 'internal_recirculation_ratio') { yMin.value = 1; yMax.value = 4 }
  if (paramY.value === 'srt') { yMin.value = 5; yMax.value = 30 }
})

const runAnalysis1d = async () => {
  loading1d.value = true
  try {
    const res = await simulationApi.sensitivity({
      param_name: param1d.value,
      min_value: min1d.value,
      max_value: max1d.value,
      steps: steps1d.value,
      influent: props.influent,
      process_config: props.processConfig,
      sim_config: props.simConfig
    })
    if (res.success) {
      result1d.value = res.data
      ElMessage.success('分析完成')
    } else {
      ElMessage.error(res.message)
    }
  } catch (e) {
    ElMessage.error('分析失败: ' + e.message)
  } finally {
    loading1d.value = false
  }
}

const runAnalysis2d = async () => {
  if (paramX.value === paramY.value) {
    ElMessage.warning('X和Y参数不能相同')
    return
  }
  loading2d.value = true
  try {
    const res = await simulationApi.sensitivity2d({
      param_x: paramX.value,
      param_y: paramY.value,
      x_min: xMin.value,
      x_max: xMax.value,
      x_steps: xSteps.value,
      y_min: yMin.value,
      y_max: yMax.value,
      y_steps: ySteps.value,
      target_param: targetParam.value,
      influent: props.influent,
      process_config: props.processConfig,
      sim_config: props.simConfig
    })
    if (res.success) {
      result2d.value = res.data
      ElMessage.success('二维分析完成')
    } else {
      ElMessage.error(res.message)
    }
  } catch (e) {
    ElMessage.error('分析失败: ' + e.message)
  } finally {
    loading2d.value = false
  }
}

const sensitivity1dOption = computed(() => {
  if (!result1d.value) return {}
  const points = result1d.value.points
  return {
    tooltip: { trigger: 'axis' },
    legend: { data: ['TN', 'NH3-N', 'TP', 'kWh/m³'], top: 0 },
    grid: { left: 60, right: 60, top: 40, bottom: 60 },
    xAxis: {
      type: 'category',
      data: points.map(p => p.param_value),
      name: param1dLabels[result1d.value.param_name] || result1d.value.param_name,
      nameLocation: 'middle',
      nameGap: 40
    },
    yAxis: [
      {
        type: 'value',
        name: '浓度 (mg/L)',
        position: 'left',
        min: 0
      },
      {
        type: 'value',
        name: 'kWh/m³',
        position: 'right',
        min: 0
      }
    ],
    series: [
      {
        name: 'TN',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: points.map(p => Number(p.tn.toFixed(2))),
        itemStyle: { color: '#67c23a' },
        lineStyle: { width: 2 }
      },
      {
        name: 'NH3-N',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: points.map(p => Number(p.nh3_n.toFixed(2))),
        itemStyle: { color: '#f56c6c' },
        lineStyle: { width: 2 }
      },
      {
        name: 'TP',
        type: 'line',
        smooth: true,
        showSymbol: false,
        data: points.map(p => Number(p.tp.toFixed(3))),
        itemStyle: { color: '#e6a23c' },
        lineStyle: { width: 2 }
      },
      {
        name: 'kWh/m³',
        type: 'line',
        smooth: true,
        showSymbol: false,
        yAxisIndex: 1,
        data: points.map(p => Number(p.kwh_per_m3.toFixed(4))),
        itemStyle: { color: '#667eea' },
        lineStyle: { width: 2, type: 'dashed' }
      }
    ]
  }
})

const sensitivity2dOption = computed(() => {
  if (!result2d.value) return {}
  const r = result2d.value
  const heatData = []
  let maxZ = -Infinity
  let minZ = Infinity
  for (let j = 0; j < r.y_values.length; j++) {
    for (let i = 0; i < r.x_values.length; i++) {
      const z = r.z_matrix[j][i]
      heatData.push([i, j, z])
      maxZ = Math.max(maxZ, z)
      minZ = Math.min(minZ, z)
    }
  }

  return {
    tooltip: {
      position: 'top',
      formatter: (p) => {
        const [xi, yi, z] = p.data
        return `${r.param_x}: ${r.x_values[xi]}<br/>${r.param_y}: ${r.y_values[yi]}<br/>${targetParamLabels[r.target_param]}: ${z.toFixed(3)}`
      }
    },
    grid: { left: 80, right: 80, top: 30, bottom: 80 },
    xAxis: {
      type: 'category',
      data: r.x_values.map(v => v.toFixed(1)),
      name: param1dLabels[r.param_x] || r.param_x,
      nameLocation: 'middle',
      nameGap: 40,
      splitArea: { show: true }
    },
    yAxis: {
      type: 'category',
      data: r.y_values.map(v => v.toFixed(1)),
      name: param1dLabels[r.param_y] || r.param_y,
      nameLocation: 'middle',
      nameGap: 60,
      splitArea: { show: true }
    },
    visualMap: {
      min: minZ,
      max: maxZ,
      calculable: true,
      orient: 'horizontal',
      left: 'center',
      bottom: 10,
      inRange: {
        color: ['#67c23a', '#e6a23c', '#f56c6c']
      }
    },
    series: [
      {
        name: targetParamLabels[r.target_param],
        type: 'heatmap',
        data: heatData,
        label: {
          show: true,
          formatter: (p) => p.data[2].toFixed(2),
          fontSize: 10
        },
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowColor: 'rgba(0, 0, 0, 0.5)'
          }
        }
      }
    ]
  }
})
</script>

<style scoped>
.sensitivity-analysis {
  width: 100%;
}
.chart-container {
  width: 100%;
  height: 360px;
}
</style>
