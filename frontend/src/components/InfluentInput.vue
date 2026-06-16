<template>
  <div class="influent-input">
    <div class="form-section">
      <div class="form-section-title">进水水质参数</div>
      <el-tabs v-model="activeTab" class="tabs-wrapper">
        <el-tab-pane label="手动输入" name="manual">
          <el-form :model="form" label-width="110px" label-position="left" size="default">
            <el-form-item
              v-for="field in fields"
              :key="field.key"
              :label="field.label"
              :error="getFieldWarning(field.key)"
            >
              <el-input-number
                v-model="form[field.key]"
                :min="0"
                :max="field.max"
                :step="field.step"
                :controls="false"
                style="width: 100%"
              />
              <template #append>{{ field.unit }}</template>
            </el-form-item>
          </el-form>
          <el-alert
            v-if="warnings.length > 0"
            type="warning"
            :closable="false"
            show-icon
            style="margin-top: 12px"
          >
            <template #title>
              <div style="font-size: 12px">
                <div v-for="(w, i) in warnings" :key="i">{{ w }}</div>
              </div>
            </template>
          </el-alert>
        </el-tab-pane>

        <el-tab-pane label="CSV导入" name="csv">
          <el-upload
            drag
            :auto-upload="false"
            :on-change="handleFileChange"
            accept=".csv"
            :limit="1"
          >
            <el-icon class="el-icon--upload"><upload-filled /></el-icon>
            <div class="el-upload__text">拖放CSV文件到此处，或<em>点击选择</em></div>
            <template #tip>
              <div class="el-upload__tip">
                CSV列: date, cod, bod5, ss, tn, nh3_n, tp, ph, temperature
              </div>
            </template>
          </el-upload>

          <div v-if="csvStatistics" style="margin-top: 20px">
            <el-alert
              type="success"
              :closable="false"
              show-icon
              style="margin-bottom: 16px"
            >
              成功解析 {{ csvStatistics.count }} 条数据
            </el-alert>

            <el-descriptions :column="2" border size="small">
              <el-descriptions-item label="指标">
                <b>均值</b> / <b>最大</b> / <b>最小</b> / <b>标准差</b>
              </el-descriptions-item>
              <el-descriptions-item label="COD (mg/L)">
                {{ fmt(csvStatistics.means.cod) }} / {{ fmt(csvStatistics.maxes.cod) }} /
                {{ fmt(csvStatistics.mins.cod) }} / {{ fmt(csvStatistics.std_devs.cod) }}
              </el-descriptions-item>
              <el-descriptions-item label="BOD5 (mg/L)">
                {{ fmt(csvStatistics.means.bod5) }} / {{ fmt(csvStatistics.maxes.bod5) }} /
                {{ fmt(csvStatistics.mins.bod5) }} / {{ fmt(csvStatistics.std_devs.bod5) }}
              </el-descriptions-item>
              <el-descriptions-item label="SS (mg/L)">
                {{ fmt(csvStatistics.means.ss) }} / {{ fmt(csvStatistics.maxes.ss) }} /
                {{ fmt(csvStatistics.mins.ss) }} / {{ fmt(csvStatistics.std_devs.ss) }}
              </el-descriptions-item>
              <el-descriptions-item label="TN (mg/L)">
                {{ fmt(csvStatistics.means.tn) }} / {{ fmt(csvStatistics.maxes.tn) }} /
                {{ fmt(csvStatistics.mins.tn) }} / {{ fmt(csvStatistics.std_devs.tn) }}
              </el-descriptions-item>
              <el-descriptions-item label="NH3-N (mg/L)">
                {{ fmt(csvStatistics.means.nh3_n) }} / {{ fmt(csvStatistics.maxes.nh3_n) }} /
                {{ fmt(csvStatistics.mins.nh3_n) }} / {{ fmt(csvStatistics.std_devs.nh3_n) }}
              </el-descriptions-item>
              <el-descriptions-item label="TP (mg/L)">
                {{ fmt(csvStatistics.means.tp) }} / {{ fmt(csvStatistics.maxes.tp) }} /
                {{ fmt(csvStatistics.mins.tp) }} / {{ fmt(csvStatistics.std_devs.tp) }}
              </el-descriptions-item>
              <el-descriptions-item label="pH">
                {{ fmt2(csvStatistics.means.ph) }} / {{ fmt2(csvStatistics.maxes.ph) }} /
                {{ fmt2(csvStatistics.mins.ph) }} / {{ fmt2(csvStatistics.std_devs.ph) }}
              </el-descriptions-item>
              <el-descriptions-item label="水温 (℃)">
                {{ fmt(csvStatistics.means.temperature) }} / {{ fmt(csvStatistics.maxes.temperature) }} /
                {{ fmt(csvStatistics.mins.temperature) }} / {{ fmt(csvStatistics.std_devs.temperature) }}
              </el-descriptions-item>
            </el-descriptions>

            <div style="margin-top: 16px">
              <div class="form-section-title">COD分布直方图</div>
              <v-chart class="chart-container" :option="histogramOption(csvStatistics.histograms.cod)" autoresize />
            </div>

            <el-button
              type="primary"
              size="default"
              style="margin-top: 16px; width: 100%"
              @click="applyMeanValues"
            >
              使用均值作为进水参数
            </el-button>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { UploadFilled } from '@element-plus/icons-vue'
import VChart from 'vue-echarts'
import { simulationApi } from '../api'
import { ElMessage } from 'element-plus'

const emit = defineEmits(['update:modelValue'])
const props = defineProps({
  modelValue: {
    type: Object,
    required: true
  }
})

const activeTab = ref('manual')
const csvStatistics = ref(null)

const form = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const fields = [
  { key: 'cod', label: 'COD', unit: 'mg/L', min: 0, max: 2000, step: 1 },
  { key: 'bod5', label: 'BOD5', unit: 'mg/L', min: 0, max: 1000, step: 1 },
  { key: 'ss', label: 'SS', unit: 'mg/L', min: 0, max: 1000, step: 1 },
  { key: 'tn', label: 'TN', unit: 'mg/L', min: 0, max: 200, step: 1 },
  { key: 'nh3_n', label: 'NH3-N', unit: 'mg/L', min: 0, max: 200, step: 1 },
  { key: 'tp', label: 'TP', unit: 'mg/L', min: 0, max: 50, step: 0.1 },
  { key: 'ph', label: 'pH', unit: '', min: 0, max: 14, step: 0.1 },
  { key: 'temperature', label: '水温', unit: '℃', min: 0, max: 50, step: 0.5 }
]

const typicalRanges = {
  cod: [100, 500],
  bod5: [50, 300],
  ss: [50, 300],
  tn: [20, 80],
  nh3_n: [10, 50],
  tp: [2, 10],
  ph: [6, 9],
  temperature: [5, 35]
}

const warnings = computed(() => {
  const result = []
  for (const field of fields) {
    const [min, max] = typicalRanges[field.key]
    const val = form.value[field.key]
    if (val < min || val > max) {
      result.push(`${field.label}值${val}${field.unit}超出典型范围${min}-${max}${field.unit}`)
    }
  }
  return result
})

const getFieldWarning = (key) => {
  const [min, max] = typicalRanges[key]
  const val = form.value[key]
  if (val < min || val > max) {
    return '超出典型范围'
  }
  return ''
}

const handleFileChange = async (uploadFile) => {
  try {
    const text = await uploadFile.raw.text()
    const result = await simulationApi.parseCsv(text)
    if (result.success) {
      csvStatistics.value = result.data.statistics
      ElMessage.success(result.message)
    } else {
      ElMessage.error(result.message)
    }
  } catch (e) {
    ElMessage.error('文件解析失败: ' + e.message)
  }
}

const applyMeanValues = () => {
  if (csvStatistics.value) {
    emit('update:modelValue', { ...csvStatistics.value.means })
    activeTab.value = 'manual'
    ElMessage.success('已应用均值')
  }
}

const histogramOption = (data) => {
  const barWidth = (data[data.length - 1][0] - data[0][0]) / data.length * 0.8
  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params) => `区间中心: ${params[0].name}<br/>频数: ${params[0].value}`
    },
    grid: { left: 50, right: 20, top: 20, bottom: 40 },
    xAxis: {
      type: 'category',
      data: data.map(d => d[0].toFixed(1)),
      name: '浓度 (mg/L)',
      nameLocation: 'middle',
      nameGap: 25
    },
    yAxis: {
      type: 'value',
      name: '频数'
    },
    series: [{
      type: 'bar',
      data: data.map(d => d[1]),
      barWidth: barWidth,
      itemStyle: {
        color: '#667eea'
      }
    }]
  }
}

const fmt = (v) => v.toFixed(1)
const fmt2 = (v) => v.toFixed(2)
</script>

<style scoped>
.influent-input {
  width: 100%;
}
.chart-container {
  width: 100%;
  height: 200px;
}
</style>
