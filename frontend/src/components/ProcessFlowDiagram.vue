<template>
  <div class="process-flow-diagram">
    <div class="diagram-container" ref="diagramContainer">
      <svg :width="svgWidth" :height="svgHeight" class="flow-svg">
        <defs>
          <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#409eff" />
          </marker>
          <marker id="arrowhead-sludge" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#e6a23c" />
          </marker>
          <marker id="arrowhead-internal" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#67c23a" />
          </marker>
          <filter id="shadow" x="-5%" y="-5%" width="110%" height="110%">
            <feDropShadow dx="0" dy="2" stdDeviation="3" flood-opacity="0.15" />
          </filter>
        </defs>

        <line
          :x1="inletX" :y1="inletY"
          :x2="anaerobic.x" :y2="anaerobic.y + unitHeight / 2"
          stroke="#409eff" stroke-width="3"
          marker-end="url(#arrowhead)"
        />
        <text :x="inletX + 10" :y="inletY - 8" class="flow-label">进水</text>
        <text :x="(inletX + anaerobic.x) / 2 - 30" :y="inletY + 18" class="flow-value">
          {{ dailyFlow.toFixed(0) }} m³/h
        </text>

        <line
          :x1="anaerobic.x + unitWidth" :y1="anaerobic.y + unitHeight / 2"
          :x2="anoxic.x" :y2="anoxic.y + unitHeight / 2"
          stroke="#409eff" stroke-width="3"
          marker-end="url(#arrowhead)"
        />
        <text :x="(anaerobic.x + unitWidth + anoxic.x) / 2 - 30" :y="anaerobic.y + unitHeight / 2 + 18" class="flow-value">
          {{ mixedFlow.toFixed(0) }} m³/h
        </text>

        <line
          :x1="anoxic.x + unitWidth" :y1="anoxic.y + unitHeight / 2"
          :x2="aerobic.x" :y2="aerobic.y + unitHeight / 2"
          stroke="#409eff" stroke-width="3"
          marker-end="url(#arrowhead)"
        />
        <text :x="(anoxic.x + unitWidth + aerobic.x) / 2 - 30" :y="anoxic.y + unitHeight / 2 + 18" class="flow-value">
          {{ mixedFlow.toFixed(0) }} m³/h
        </text>

        <line
          :x1="aerobic.x + unitWidth" :y1="aerobic.y + unitHeight / 2"
          :x2="clarifier.x" :y2="clarifier.y + unitHeight / 2"
          stroke="#409eff" stroke-width="3"
          marker-end="url(#arrowhead)"
        />
        <text :x="(aerobic.x + unitWidth + clarifier.x) / 2 - 30" :y="aerobic.y + unitHeight / 2 + 18" class="flow-value">
          {{ mixedFlow.toFixed(0) }} m³/h
        </text>

        <line
          :x1="clarifier.x + unitWidth" :y1="clarifier.y + unitHeight / 2"
          :x2="outletX" :y2="outletY"
          stroke="#409eff" stroke-width="3"
          marker-end="url(#arrowhead)"
        />
        <text :x="outletX + 10" :y="outletY - 8" class="flow-label">出水</text>
        <text :x="(clarifier.x + unitWidth + outletX) / 2 - 30" :y="outletY + 18" class="flow-value">
          {{ dailyFlow.toFixed(0) }} m³/h
        </text>

        <path
          :d="internalRecirculationPath"
          stroke="#67c23a" stroke-width="2.5"
          stroke-dasharray="6 3"
          fill="none"
          marker-end="url(#arrowhead-internal)"
        />
        <text
          :x="(aerobic.x + unitWidth + anoxic.x) / 2 - 30"
          :y="topRowY + 20"
          class="flow-label-internal"
        >
          内回流 {{ (internalRatio * 100).toFixed(0) }}%
        </text>
        <text
          :x="(aerobic.x + unitWidth + anoxic.x) / 2 - 35"
          :y="topRowY + 36"
          class="flow-value-small"
        >
          {{ internalFlow.toFixed(0) }} m³/h
        </text>

        <path
          :d="sludgeRecirculationPath"
          stroke="#e6a23c" stroke-width="2.5"
          stroke-dasharray="6 3"
          fill="none"
          marker-end="url(#arrowhead-sludge)"
        />
        <text
          :x="(clarifier.x + unitWidth + anaerobic.x) / 2 - 30"
          :y="bottomRowY + 30"
          class="flow-label-sludge"
        >
          污泥回流 {{ (sludgeRatio * 100).toFixed(0) }}%
        </text>
        <text
          :x="(clarifier.x + unitWidth + anaerobic.x) / 2 - 35"
          :y="bottomRowY + 46"
          class="flow-value-small"
        >
          {{ sludgeFlow.toFixed(0) }} m³/h
        </text>

        <g
          class="unit-group anaerobic-group"
          @mouseenter="showTooltip('anaerobic', $event)"
          @mouseleave="hideTooltip"
        >
          <rect
            :x="anaerobic.x" :y="anaerobic.y"
            :width="unitWidth" :height="unitHeight"
            rx="8" ry="8"
            class="unit-rect unit-anaerobic"
            filter="url(#shadow)"
          />
          <text :x="anaerobic.x + unitWidth / 2" :y="anaerobic.y + 26" class="unit-title">厌氧池</text>
          <line :x1="anaerobic.x + 10" :y1="anaerobic.y + 38" :x2="anaerobic.x + unitWidth - 10" :y2="anaerobic.y + 38" class="unit-divider" />
          <text :x="anaerobic.x + unitWidth / 2" :y="anaerobic.y + 58" class="unit-param">COD: {{ displayData.anaerobic.cod }}</text>
          <text :x="anaerobic.x + unitWidth / 2" :y="anaerobic.y + 76" class="unit-param">NH3-N: {{ displayData.anaerobic.nh3_n }}</text>
          <text :x="anaerobic.x + unitWidth / 2" :y="anaerobic.y + 94" class="unit-param">DO: {{ displayData.anaerobic.do }}</text>
        </g>

        <g
          class="unit-group anoxic-group"
          @mouseenter="showTooltip('anoxic', $event)"
          @mouseleave="hideTooltip"
        >
          <rect
            :x="anoxic.x" :y="anoxic.y"
            :width="unitWidth" :height="unitHeight"
            rx="8" ry="8"
            class="unit-rect unit-anoxic"
            filter="url(#shadow)"
          />
          <text :x="anoxic.x + unitWidth / 2" :y="anoxic.y + 26" class="unit-title">缺氧池</text>
          <line :x1="anoxic.x + 10" :y1="anoxic.y + 38" :x2="anoxic.x + unitWidth - 10" :y2="anoxic.y + 38" class="unit-divider" />
          <text :x="anoxic.x + unitWidth / 2" :y="anoxic.y + 58" class="unit-param">COD: {{ displayData.anoxic.cod }}</text>
          <text :x="anoxic.x + unitWidth / 2" :y="anoxic.y + 76" class="unit-param">NH3-N: {{ displayData.anoxic.nh3_n }}</text>
          <text :x="anoxic.x + unitWidth / 2" :y="anoxic.y + 94" class="unit-param">DO: {{ displayData.anoxic.do }}</text>
        </g>

        <g
          class="unit-group aerobic-group"
          @mouseenter="showTooltip('aerobic', $event)"
          @mouseleave="hideTooltip"
        >
          <rect
            :x="aerobic.x" :y="aerobic.y"
            :width="unitWidth" :height="unitHeight"
            rx="8" ry="8"
            class="unit-rect unit-aerobic"
            filter="url(#shadow)"
          />
          <text :x="aerobic.x + unitWidth / 2" :y="aerobic.y + 26" class="unit-title">好氧池</text>
          <line :x1="aerobic.x + 10" :y1="aerobic.y + 38" :x2="aerobic.x + unitWidth - 10" :y2="aerobic.y + 38" class="unit-divider" />
          <text :x="aerobic.x + unitWidth / 2" :y="aerobic.y + 58" class="unit-param">COD: {{ displayData.aerobic.cod }}</text>
          <text :x="aerobic.x + unitWidth / 2" :y="aerobic.y + 76" class="unit-param">NH3-N: {{ displayData.aerobic.nh3_n }}</text>
          <text :x="aerobic.x + unitWidth / 2" :y="aerobic.y + 94" class="unit-param">DO: {{ displayData.aerobic.do }}</text>
        </g>

        <g
          class="unit-group clarifier-group"
          @mouseenter="showTooltip('clarifier', $event)"
          @mouseleave="hideTooltip"
        >
          <rect
            :x="clarifier.x" :y="clarifier.y"
            :width="unitWidth" :height="unitHeight"
            rx="8" ry="8"
            class="unit-rect unit-clarifier"
            filter="url(#shadow)"
          />
          <text :x="clarifier.x + unitWidth / 2" :y="clarifier.y + 26" class="unit-title">二沉池</text>
          <line :x1="clarifier.x + 10" :y1="clarifier.y + 38" :x2="clarifier.x + unitWidth - 10" :y2="clarifier.y + 38" class="unit-divider" />
          <text :x="clarifier.x + unitWidth / 2" :y="clarifier.y + 58" class="unit-param">COD: {{ displayData.clarifier.cod }}</text>
          <text :x="clarifier.x + unitWidth / 2" :y="clarifier.y + 76" class="unit-param">NH3-N: {{ displayData.clarifier.nh3_n }}</text>
          <text :x="clarifier.x + unitWidth / 2" :y="clarifier.y + 94" class="unit-param">DO: {{ displayData.clarifier.do }}</text>
        </g>
      </svg>

      <transition name="fade">
        <div
          v-if="tooltipVisible"
          class="unit-tooltip"
          :style="tooltipStyle"
        >
          <div class="tooltip-header">{{ tooltipData.name }}</div>
          <div class="tooltip-body">
            <div class="tooltip-row" v-for="item in tooltipData.items" :key="item.label">
              <span class="tooltip-label">{{ item.label }}</span>
              <span class="tooltip-value">{{ item.value }}{{ item.unit }}</span>
            </div>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, reactive } from 'vue'

const props = defineProps({
  timeSeriesStep: {
    type: Object,
    default: null
  },
  unitEffluents: {
    type: Object,
    default: null
  },
  processConfig: {
    type: Object,
    required: true
  }
})

const diagramContainer = ref(null)
const tooltipVisible = ref(false)
const tooltipStyle = reactive({ left: '0px', top: '0px' })
const tooltipData = reactive({ name: '', items: [] })

const svgWidth = 900
const svgHeight = 380
const unitWidth = 140
const unitHeight = 110
const unitGap = 50

const centerY = svgHeight / 2 - unitHeight / 2
const startX = 80

const anaerobic = reactive({ x: startX, y: centerY })
const anoxic = reactive({ x: startX + unitWidth + unitGap, y: centerY })
const aerobic = reactive({ x: startX + (unitWidth + unitGap) * 2, y: centerY })
const clarifier = reactive({ x: startX + (unitWidth + unitGap) * 3, y: centerY })

const inletX = 20
const inletY = centerY + unitHeight / 2
const outletX = svgWidth - 20
const outletY = centerY + unitHeight / 2

const topRowY = centerY - 20
const bottomRowY = centerY + unitHeight + 20

const dailyFlow = computed(() => props.processConfig.daily_flow / 24)
const sludgeRatio = computed(() => props.processConfig.sludge_recirculation_ratio)
const internalRatio = computed(() => props.processConfig.internal_recirculation_ratio)
const sludgeFlow = computed(() => dailyFlow.value * sludgeRatio.value)
const internalFlow = computed(() => dailyFlow.value * internalRatio.value)
const mixedFlow = computed(() => dailyFlow.value * (1 + sludgeRatio.value + internalRatio.value))

const internalRecirculationPath = computed(() => {
  const startX = aerobic.x + unitWidth * 0.8
  const startY = aerobic.y
  const endX = anoxic.x + unitWidth * 0.2
  const endY = anoxic.y
  const midY = topRowY - 10
  return `M ${startX} ${startY} L ${startX} ${midY} L ${endX} ${midY} L ${endX} ${endY}`
})

const sludgeRecirculationPath = computed(() => {
  const startX = clarifier.x + unitWidth * 0.2
  const startY = clarifier.y + unitHeight
  const endX = anaerobic.x + unitWidth * 0.8
  const endY = anaerobic.y + unitHeight
  const midY = bottomRowY + 50
  return `M ${startX} ${startY} L ${startX} ${midY} L ${endX} ${midY} L ${endX} ${endY}`
})

const formatValue = (v, digits = 1) => {
  if (v === undefined || v === null || isNaN(v)) return '-'
  return Number(v).toFixed(digits)
}

const displayData = computed(() => {
  let data = {
    anaerobic: { cod: '-', nh3_n: '-', do: '-' },
    anoxic: { cod: '-', nh3_n: '-', do: '-' },
    aerobic: { cod: '-', nh3_n: '-', do: '-' },
    clarifier: { cod: '-', nh3_n: '-', do: '-' }
  }

  const source = props.timeSeriesStep || props.unitEffluents

  if (source) {
    if (source.anaerobic_effluent) {
      data.anaerobic.cod = formatValue(source.anaerobic_effluent.cod)
      data.anaerobic.nh3_n = formatValue(source.anaerobic_effluent.nh3_n, 2)
      data.anaerobic.do = formatValue(props.processConfig.anaerobic.do_setpoint, 2)
    } else if (source.anaerobic) {
      data.anaerobic.cod = formatValue(source.anaerobic.cod)
      data.anaerobic.nh3_n = formatValue(source.anaerobic.nh3_n, 2)
      data.anaerobic.do = formatValue(props.processConfig.anaerobic.do_setpoint, 2)
    }

    if (source.anoxic_effluent) {
      data.anoxic.cod = formatValue(source.anoxic_effluent.cod)
      data.anoxic.nh3_n = formatValue(source.anoxic_effluent.nh3_n, 2)
      data.anoxic.do = formatValue(props.processConfig.anoxic.do_setpoint, 2)
    } else if (source.anoxic) {
      data.anoxic.cod = formatValue(source.anoxic.cod)
      data.anoxic.nh3_n = formatValue(source.anoxic.nh3_n, 2)
      data.anoxic.do = formatValue(props.processConfig.anoxic.do_setpoint, 2)
    }

    if (source.aerobic_effluent) {
      data.aerobic.cod = formatValue(source.aerobic_effluent.cod)
      data.aerobic.nh3_n = formatValue(source.aerobic_effluent.nh3_n, 2)
      data.aerobic.do = formatValue(props.processConfig.aerobic.do_setpoint, 2)
    } else if (source.aerobic) {
      data.aerobic.cod = formatValue(source.aerobic.cod)
      data.aerobic.nh3_n = formatValue(source.aerobic.nh3_n, 2)
      data.aerobic.do = formatValue(props.processConfig.aerobic.do_setpoint, 2)
    }

    if (source.final_effluent) {
      data.clarifier.cod = formatValue(source.final_effluent.cod)
      data.clarifier.nh3_n = formatValue(source.final_effluent.nh3_n, 2)
      data.clarifier.do = formatValue(props.processConfig.clarifier.do_setpoint, 2)
    } else if (source.clarifier) {
      data.clarifier.cod = formatValue(source.clarifier.cod)
      data.clarifier.nh3_n = formatValue(source.clarifier.nh3_n, 2)
      data.clarifier.do = formatValue(props.processConfig.clarifier.do_setpoint, 2)
    }
  }

  return data
})

const unitNames = {
  anaerobic: '厌氧池',
  anoxic: '缺氧池',
  aerobic: '好氧池',
  clarifier: '二沉池'
}

const showTooltip = (unit, event) => {
  const source = props.timeSeriesStep || props.unitEffluents
  if (!source) return

  let effluent = null
  if (source[unit + '_effluent']) {
    effluent = source[unit + '_effluent']
  } else if (source[unit]) {
    effluent = source[unit]
  }

  if (!effluent) return

  const doValue = props.processConfig[unit]?.do_setpoint ?? 0

  tooltipData.name = unitNames[unit]
  tooltipData.items = [
    { label: 'COD', value: formatValue(effluent.cod), unit: ' mg/L' },
    { label: 'BOD5', value: formatValue(effluent.bod5), unit: ' mg/L' },
    { label: 'SS', value: formatValue(effluent.ss), unit: ' mg/L' },
    { label: 'TN', value: formatValue(effluent.tn), unit: ' mg/L' },
    { label: 'NH3-N', value: formatValue(effluent.nh3_n, 2), unit: ' mg/L' },
    { label: 'NO3-N', value: formatValue(effluent.no3_n, 2), unit: ' mg/L' },
    { label: 'TP', value: formatValue(effluent.tp, 2), unit: ' mg/L' },
    { label: 'DO', value: formatValue(doValue, 2), unit: ' mg/L' }
  ]

  const rect = diagramContainer.value.getBoundingClientRect()
  tooltipStyle.left = (event.offsetX + 15) + 'px'
  tooltipStyle.top = (event.offsetY - 10) + 'px'
  tooltipVisible.value = true
}

const hideTooltip = () => {
  tooltipVisible.value = false
}
</script>

<style scoped>
.process-flow-diagram {
  width: 100%;
  overflow-x: auto;
}

.diagram-container {
  position: relative;
  display: flex;
  justify-content: center;
  padding: 10px;
  background: linear-gradient(180deg, #f0f9ff 0%, #f0fdf4 100%);
  border-radius: 12px;
  border: 1px solid #ebeef5;
}

.flow-svg {
  display: block;
}

.unit-rect {
  stroke-width: 2;
  cursor: pointer;
  transition: all 0.3s ease;
}

.unit-anaerobic {
  fill: #fef3c7;
  stroke: #d97706;
}

.unit-anoxic {
  fill: #dbeafe;
  stroke: #2563eb;
}

.unit-aerobic {
  fill: #d1fae5;
  stroke: #059669;
}

.unit-clarifier {
  fill: #e9d5ff;
  stroke: #7c3aed;
}

.unit-group:hover .unit-rect {
  filter: url(#shadow) brightness(1.05);
  stroke-width: 3;
}

.unit-title {
  text-anchor: middle;
  font-size: 15px;
  font-weight: 700;
  fill: #1f2937;
}

.unit-divider {
  stroke: rgba(0, 0, 0, 0.1);
  stroke-width: 1;
}

.unit-param {
  text-anchor: middle;
  font-size: 12px;
  fill: #374151;
  font-family: 'Consolas', 'Monaco', monospace;
}

.flow-label {
  font-size: 13px;
  font-weight: 600;
  fill: #409eff;
}

.flow-label-internal {
  font-size: 12px;
  font-weight: 600;
  fill: #67c23a;
  text-anchor: middle;
}

.flow-label-sludge {
  font-size: 12px;
  font-weight: 600;
  fill: #e6a23c;
  text-anchor: middle;
}

.flow-value {
  font-size: 11px;
  fill: #606266;
  font-family: 'Consolas', 'Monaco', monospace;
}

.flow-value-small {
  font-size: 10px;
  fill: #909399;
  font-family: 'Consolas', 'Monaco', monospace;
  text-anchor: middle;
}

.unit-tooltip {
  position: absolute;
  z-index: 100;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  overflow: hidden;
  pointer-events: none;
  border: 1px solid #ebeef5;
}

.tooltip-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 8px 12px;
  font-weight: 600;
  font-size: 14px;
}

.tooltip-body {
  padding: 10px 12px;
}

.tooltip-row {
  display: flex;
  justify-content: space-between;
  padding: 3px 0;
  font-size: 12px;
}

.tooltip-label {
  color: #606266;
}

.tooltip-value {
  color: #303133;
  font-weight: 600;
  font-family: 'Consolas', 'Monaco', monospace;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
