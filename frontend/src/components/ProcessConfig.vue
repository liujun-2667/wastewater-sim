<template>
  <div class="process-config">
    <div class="form-section">
      <div class="form-section-title">工艺模板</div>
      <el-radio-group v-model="templateType" @change="loadTemplate" style="width: 100%">
        <el-radio-button value="small">小型(1万m³/日)</el-radio-button>
        <el-radio-button value="medium">中型(5万m³/日)</el-radio-button>
        <el-radio-button value="large">大型(20万m³/日)</el-radio-button>
        <el-radio-button value="custom">自定义</el-radio-button>
      </el-radio-group>
    </div>

    <div class="form-section">
      <div class="form-section-title">全局参数</div>
      <el-form :model="form" label-width="120px" size="default" label-position="left">
        <el-form-item label="日处理量">
          <el-input-number v-model="form.daily_flow" :min="100" :max="1000000" :step="1000" :controls="false" style="width: 100%" />
          <template #append>m³/日</template>
        </el-form-item>
        <el-form-item label="污泥回流比 R">
          <el-input-number v-model="form.sludge_recirculation_ratio" :min="0.1" :max="3" :step="0.1" :controls="false" style="width: 100%" />
          <template #append>(50%-150%)</template>
        </el-form-item>
        <el-form-item label="内回流比 r">
          <el-input-number v-model="form.internal_recirculation_ratio" :min="0.5" :max="6" :step="0.1" :controls="false" style="width: 100%" />
          <template #append>(100%-400%)</template>
        </el-form-item>
        <el-form-item label="污泥龄 SRT">
          <el-input-number v-model="form.srt" :min="1" :max="60" :step="1" :controls="false" style="width: 100%" />
          <template #append>天</template>
        </el-form-item>
        <el-form-item label="曝气量">
          <el-input-number v-model="form.aeration_rate" :min="10" :max="100000" :step="10" :controls="false" style="width: 100%" />
          <template #append>m³/h</template>
        </el-form-item>
      </el-form>
    </div>

    <div class="form-section">
      <div class="form-section-title">各单元配置</div>
      <el-tabs v-model="activeUnit" class="tabs-wrapper" type="card">
        <el-tab-pane label="厌氧池" name="anaerobic">
          <el-form :model="form.anaerobic" label-width="110px" size="default" label-position="left">
            <el-form-item label="池容">
              <el-input-number v-model="form.anaerobic.volume" :min="10" :max="1000000" :step="10" :controls="false" style="width: 100%" />
              <template #append>m³</template>
            </el-form-item>
            <el-form-item label="HRT">
              <el-input-number v-model="form.anaerobic.hrt" :min="0.1" :max="48" :step="0.1" :controls="false" style="width: 100%" />
              <template #append>h</template>
            </el-form-item>
            <el-form-item label="MLSS">
              <el-input-number v-model="form.anaerobic.mlss" :min="500" :max="10000" :step="100" :controls="false" style="width: 100%" />
              <template #append>mg/L</template>
            </el-form-item>
            <el-form-item label="DO设定">
              <el-input-number v-model="form.anaerobic.do_setpoint" :min="0" :max="0.5" :step="0.05" :controls="false" style="width: 100%" />
              <template #append>(<0.2mg/L)</template>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="缺氧池" name="anoxic">
          <el-form :model="form.anoxic" label-width="110px" size="default" label-position="left">
            <el-form-item label="池容">
              <el-input-number v-model="form.anoxic.volume" :min="10" :max="1000000" :step="10" :controls="false" style="width: 100%" />
              <template #append>m³</template>
            </el-form-item>
            <el-form-item label="HRT">
              <el-input-number v-model="form.anoxic.hrt" :min="0.1" :max="48" :step="0.1" :controls="false" style="width: 100%" />
              <template #append>h</template>
            </el-form-item>
            <el-form-item label="MLSS">
              <el-input-number v-model="form.anoxic.mlss" :min="500" :max="10000" :step="100" :controls="false" style="width: 100%" />
              <template #append>mg/L</template>
            </el-form-item>
            <el-form-item label="DO设定">
              <el-input-number v-model="form.anoxic.do_setpoint" :min="0" :max="1" :step="0.05" :controls="false" style="width: 100%" />
              <template #append>(<0.5mg/L)</template>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="好氧池" name="aerobic">
          <el-form :model="form.aerobic" label-width="110px" size="default" label-position="left">
            <el-form-item label="池容">
              <el-input-number v-model="form.aerobic.volume" :min="10" :max="1000000" :step="10" :controls="false" style="width: 100%" />
              <template #append>m³</template>
            </el-form-item>
            <el-form-item label="HRT">
              <el-input-number v-model="form.aerobic.hrt" :min="0.1" :max="48" :step="0.1" :controls="false" style="width: 100%" />
              <template #append>h</template>
            </el-form-item>
            <el-form-item label="MLSS">
              <el-input-number v-model="form.aerobic.mlss" :min="500" :max="10000" :step="100" :controls="false" style="width: 100%" />
              <template #append>mg/L</template>
            </el-form-item>
            <el-form-item label="DO设定">
              <el-input-number v-model="form.aerobic.do_setpoint" :min="0.5" :max="6" :step="0.1" :controls="false" style="width: 100%" />
              <template #append>(2-4mg/L)</template>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="二沉池" name="clarifier">
          <el-form :model="form.clarifier" label-width="110px" size="default" label-position="left">
            <el-form-item label="池容">
              <el-input-number v-model="form.clarifier.volume" :min="10" :max="1000000" :step="10" :controls="false" style="width: 100%" />
              <template #append>m³</template>
            </el-form-item>
            <el-form-item label="HRT">
              <el-input-number v-model="form.clarifier.hrt" :min="0.1" :max="48" :step="0.1" :controls="false" style="width: 100%" />
              <template #append>h</template>
            </el-form-item>
            <el-form-item label="污泥浓度">
              <el-input-number v-model="form.clarifier.mlss" :min="1000" :max="20000" :step="100" :controls="false" style="width: 100%" />
              <template #append>mg/L</template>
            </el-form-item>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { simulationApi } from '../api'
import { ElMessage } from 'element-plus'

const emit = defineEmits(['update:modelValue'])
const props = defineProps({
  modelValue: {
    type: Object,
    required: true
  }
})

const templateType = ref('medium')
const activeUnit = ref('anaerobic')
const templates = ref({})

onMounted(async () => {
  try {
    const res = await simulationApi.getTemplates()
    if (res.success && res.data) {
      templates.value = res.data
    }
  } catch (e) {
    console.warn('模板加载失败', e)
  }
})

const form = computed({
  get: () => props.modelValue,
  set: (val) => {
    templateType.value = 'custom'
    emit('update:modelValue', val)
  }
})

const loadTemplate = (type) => {
  if (type === 'custom') return
  const tplMap = {
    small: templates.value.small,
    medium: templates.value.medium,
    large: templates.value.large
  }
  const tpl = tplMap[type]
  if (tpl) {
    emit('update:modelValue', JSON.parse(JSON.stringify(tpl)))
    ElMessage.success(`已加载${type === 'small' ? '小型' : type === 'medium' ? '中型' : '大型'}污水厂模板`)
  }
}
</script>

<style scoped>
.process-config {
  width: 100%;
}
</style>
