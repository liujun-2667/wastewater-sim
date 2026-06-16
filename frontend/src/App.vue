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

            <el-button
              type="primary"
              size="large"
              style="width: 100%"
              class="btn-primary"
              :loading="simulating"
              @click="runSimulation"
            >
              <el-icon style="margin-right: 6px"><video-play /></el-icon>
              {{ simulating ? '仿真计算中...' : '开始仿真' }}
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
                <SimulationResults :result="simulationResult" />
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
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { Setting, DataAnalysis, Aim, VideoPlay } from '@element-plus/icons-vue'
import InfluentInput from './components/InfluentInput.vue'
import ProcessConfig from './components/ProcessConfig.vue'
import SimulationResults from './components/SimulationResults.vue'
import SensitivityAnalysis from './components/SensitivityAnalysis.vue'
import { simulationApi } from './api'
import { ElMessage } from 'element-plus'

const activeResultTab = ref('results')
const simulating = ref(false)
const simulationResult = ref(null)

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

onMounted(async () => {
  try {
    const res = await simulationApi.getTemplates()
    if (res.success && res.data && res.data.medium) {
      Object.assign(processConfig, JSON.parse(JSON.stringify(res.data.medium)))
    }
  } catch (e) {
    console.warn('模板加载失败，使用默认值', e)
  }
})

const runSimulation = async () => {
  simulating.value = true
  try {
    const res = await simulationApi.simulate({
      influent: { ...influent },
      process_config: JSON.parse(JSON.stringify(processConfig)),
      sim_config: { ...simConfig }
    })

    if (res.data) {
      simulationResult.value = res.data
      if (res.data.success) {
        ElMessage.success('仿真完成')
      } else {
        ElMessage.warning(res.data.message)
      }
    } else {
      ElMessage.error(res.message || '仿真失败')
    }
  } catch (e) {
    ElMessage.error('仿真失败: ' + (e.message || '未知错误'))
  } finally {
    simulating.value = false
  }
}
</script>

<style scoped>
.left-panel {
  position: sticky;
  top: 20px;
}
</style>
