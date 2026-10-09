<script setup lang="ts">
import { ref, onMounted, onBeforeMount } from "vue";
const { invoke, Channel } = window.__TAURI__.core;
const { message } = window.__TAURI__.dialog;
import * as echarts from 'echarts';

import en from './locales/en.js';
import zh from './locales/zh.js';
import ja from './locales/ja.js';
import ru from './locales/ru.js';

const locales = { en, zh, ja, ru };
let currentLang = 'zh'; // 默认中文

function t(key) {
  return locales[currentLang][key] || key;
}

async function initLanguage() {
  const savedLang = localStorage.getItem('appLanguage') || 'zh';
  await applyLanguage(savedLang);
}

// 应用语言设置
async function applyLanguage(lang) {
  currentLang = lang;
  localStorage.setItem('appLanguage', lang);

  // 更新按钮高亮状态
  document.querySelectorAll('.lang-btn').forEach(btn => {
    btn.classList.toggle('active',
      (btn.id === 'lang-en' && lang === 'en') ||
      (btn.id === 'lang-zh' && lang === 'zh') ||
      (btn.id === 'lang-ja' && lang === 'ja') ||
      (btn.id === 'lang-ru' && lang === 'ru')
    );
  });

  const elements = document.querySelectorAll('[data-i18n], [data-i18n-placeholder]');
  elements.forEach(el => {
    const key = el.dataset.i18n || el.dataset.i18nPlaceholder;
    if (el.dataset.i18nPlaceholder) {
      el.placeholder = t(key);
    } else {
      el.textContent = t(key);
    }
  });

  if (voltageCurrentChart) {
    const isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const newOption = getChartOption(isDarkMode);
    voltageCurrentChart.setOption(newOption);
  }
}

onMounted(async () => {
  await initLanguage(); // 只需调用一次语言初始化方法
});

// 串口配置界面

const INPUT_TYPE = {
  0x05:{
    text: "PD"
  },
  0x06:{
    text: "DC"
  },
  0x00:{
    text: "UNKNOWN"
  }
};
// 通讯协议配置
const PROTOCOL_CONFIG = {
  WHO_AM_I: {
    command: [0x81, 0x0a],
    responseHeader: 0x81,
    responseFooter: 0x0a,
    frameLength: -1,
    handler: (data) => {
      const nameBytes = data.slice(1, -1);
      return new TextDecoder().decode(nameBytes);
    }
  },
  READ_SYSTEM_VERSION: {
    command: [0xC2, 0x0a],
    responseHeader: 0xC2,
    responseFooter: 0x0a,
    frameLength: -1,
    handler: (data) => {
      const nameBytes = data.slice(1, -1);
      return new TextDecoder().decode(nameBytes);
    }
  },
  READ_SYSTEM_SERIAL_NUM: {
    command: [0xC3, 0x0a],
    responseHeader: 0xC3,
    responseFooter: 0x0a,
    frameLength: -1,
    handler: (data) => {
      const nameBytes = data.slice(1, -1);
      return new TextDecoder().decode(nameBytes);
    }
  },
  WRITE_SYSTEM_CONFIG_SAVE: {
    command: [0x44, 0x0a],
    responseHeader: 0x44,
    responseFooter: 0x0a,
    handler: (data) => {
      return {
        success: true
      };
    }
  },
  READ_OUTPUT_EN_STATE: {
    command: [0x82, 0x0a],
    responseHeader: 0x82,
    responseFooter: 0x0a,
    frameLength: -1,
    handler: (data) => {
      return {
        state: data[1]
      };
    }
  },
  WRITE_OUTPUT_ENABLE: {
    command: (enable) => [0x02, enable, 0x0a],
    responseHeader: 0x02,
    responseFooter: 0x0a,
    handler: (data) => {
      return {
        success: true
      };
    }
  },
  READ_OUTPUT_ID: {
    command: [0x83, 0x0a],
    responseHeader: 0x83,
    responseFooter: 0x0a,
    frameLength: 3,
    handler: (data) => {
      return {
        id: data[1]
      };
    }
  },
  WRITE_OUTPUT_ID: {
    command: (id) => [0x03, id, 0x0a],
    responseHeader: 0x03,
    responseFooter: 0x0a,
    handler: (data) => {
      return {
        success: true
      };
    }
  },
  READ_OUTPUT_DATA: {
    command: (id) => [0x84, id, 0x0a],
    responseHeader: 0x84,
    responseFooter: 0x0a,
    frameLength: 7,
    handler: (data) => {
      const id = data[1];
      const voltage = (data[3] << 8) | data[2];
      const current = (data[5] << 8) | data[4];
      return {
        id,
        voltage: voltage / 1000,
        current: current / 1000
      };
    }
  },
  WRITE_OUTPUT_DATA: {
    command: (id, voltage, current) => {
      return [
        0x04,
        id,
        voltage & 0xFF,         // 电压低字节
        (voltage >> 8) & 0xFF,  // 电压高字节
        current & 0xFF,         // 电流低字节
        (current >> 8) & 0xFF,  // 电流高字节
        0x0a
      ];
    },
    responseHeader: 0x04,
    responseFooter: 0x0a,
    handler: (data) => {
      return {
        success: true
      };
    }
  },
  READ_OUTPUT_DISPLAY: {
    command: [0x85, 0x0a],
    responseHeader: 0x85,
    responseFooter: 0x0a,
    frameLength: 6,
    handler: (data) => {
      const voltage = (data[2] << 8) | data[1];
      const current = (data[4] << 8) | data[3];
      return {
        voltage: voltage / 1000,
        current: current / 1000
      };
    }
  },
  READ_INPUT_STATE: {
    command: [0x8a, 0x0a],
    responseHeader: 0x8a,
    responseFooter: 0x0a,
    frameLength: 7,
    handler: (data) => {
      const state = data[1];
      const voltage = ((data[3] << 8) | data[2]);
      const pd_voltage = ((data[5] << 8) | data[4]);
      return {
        state: state,
        voltage: voltage / 1000,
        pd_voltage: pd_voltage / 10
      };
    }
  },
  WRITE_INPUT_PD_VOLTAGE: {
    command: (voltage) => {
      return [
        0x0a,
        voltage & 0xFF,         // 电压低字节
        (voltage >> 8) & 0xFF,  // 电压高字节
        0x0a
      ];
    },
    responseHeader: 0x0a,
    responseFooter: 0x0a,
    handler: (data) => {
      return {
        success: true
      };
    }
  },
};

let serialPortList;
let serialPortOpen;
var isOpen = false;
var pd_init = false;

const DEVICE_PDPowerMiniV1 = "AC";
const DEVICE_PDPowerMiniV11 = "B3";
let device_type = DEVICE_PDPowerMiniV1;

const DEVICE_RANGE_CONFIG = {
  [DEVICE_PDPowerMiniV1]: {
    voltage: { min: 0.5, max: 20},
    current: { min: 0.005, max: 3},
    pd_voltage: { min: 8, max: 24}
  },
  [DEVICE_PDPowerMiniV11]: {
    voltage: { min: 0.5, max: 24},
    current: { min: 0.005, max: 3},
    pd_voltage: { min: 8, max: 28}
  }
}

type SerialEvent = {
  event: "opened" | "closed" | "error" | "read",
  data: any
};

function SerialEventInit() {
  const onSerialEvent = new Channel<SerialEvent>();
  onSerialEvent.onmessage = async (m) => {
    // console.log(`event ${m.event} data: ${JSON.stringify(m.data)}`);
    if (m.event == "read") {
      await port_listening(m.data['data']);
    }
    else if (m.event == "error") {
      console.log(t("port_disconnected"));
      isOpen = false;
      pd_init = false;
      serialPortOpen.textContent = t("open");
      serialPortOpen.setAttribute('data-i18n', 'open');
      serialPortOpen.style.backgroundColor = "";
      serialPortList.disabled = false;
      document.getElementById("device_name").textContent = "Name:";
      document.getElementById("device_version").textContent = "Version:";
      document.getElementById("device_serial").textContent = "SN:";
      await stopPeriodicRead();
      // 可选：显示断开提示
      await message(t("port_disconnected"), {
        title: t("warning"),
        kind: 'warning'
      });
    }
  };
  return onSerialEvent;
}

onMounted(() => {
  // 在组件挂载后获取 DOM 元素
  serialPortOpen = document.getElementById('open-serial');
  serialPortList = document.getElementById('serial-port-list');
  serialPortList.addEventListener('mousedown', refreshSerialPorts);
  serialPortOpen.addEventListener('click', open_serial);

  refreshSerialPorts();
});

// 定义刷新串口列表的函数
async function refreshSerialPorts() {
  try {
    // 调用后端命令获取串口列表
    const ports = await invoke("serial_get_available_ports");
    console.log(ports);
    // 清空现有选项
    serialPortList.innerHTML = '';
    ports[0].forEach(port => {
      const option = document.createElement('option');
      option.value = port;
      option.textContent = port;
      serialPortList.appendChild(option);
    });
    serialPortList.selectedIndex = 0;
  } catch (error) {
    console.error('获取串口列表失败:', error);
  }
}

async function open_serial() {
  console.log(serialPortList.value);
  if (isOpen) {
    await invoke("serial_close_port");

    isOpen = false;
    pd_init = false;
    serialPortOpen.textContent = t("open");
    serialPortOpen.setAttribute('data-i18n', 'open');
    serialPortOpen.style.backgroundColor = "";
    serialPortList.disabled = false;
    document.getElementById("device_name").textContent = "Name:";
    document.getElementById("device_version").textContent = "Version:";
    document.getElementById("device_serial").textContent = "SN:";
    if(showInputState.value){
      document.getElementById("device_input_state").textContent = "Input Type: ";
      document.getElementById("device_input_voltage").textContent = "Input Voltage: ";
    }
    showPDSettings.value = false;
    stopPeriodicRead();
  } else {
    try {
      let success = await invoke("serial_open_port", { portName: serialPortList.value, channel: SerialEventInit() });
      if (success) {
        isOpen = true;
        serialPortOpen.textContent = t("close");
        serialPortOpen.setAttribute('data-i18n', 'close');
        serialPortOpen.style.backgroundColor = "red";
        serialPortList.disabled = true;

        // 发送获取设备信息的命令
        DeviceInfoGet();
      }
    } catch (error) {
      console.error(t("open_fail"), error);
      await message(t("open_fail") + `: ${error.message || error}`, {
        title: t("error"),
        kind: 'error',
      });
      isOpen = false;
    }
  }
}

function versionGe(verStr, targetVer) {
  // 1. 去除开头V，截断下划线前半段
  const clean = verStr.replace(/^V/i, "").split("_")[0];
  const arr = clean.split(".").map(Number);
  const targetArr = targetVer.split(".").map(Number);

  // 补齐数组长度，短的补0
  const maxLen = Math.max(arr.length, targetArr.length);
  for (let i = 0; i < maxLen; i++) {
    const a = arr[i] ?? 0;
    const t = targetArr[i] ?? 0;
    if (a > t) return true;
    if (a < t) return false;
  }
  // 全部段相等
  return true;
}

let remainingData = new Uint8Array(0);
async function port_listening(data) {
  if (!(data instanceof Uint8Array)) {
    data = new Uint8Array(data);
  }

  // 合并新数据和剩余数据
  const combinedBuffer = new Uint8Array(remainingData.length + data.length);
  combinedBuffer.set(remainingData, 0);
  combinedBuffer.set(data, remainingData.length);

  let buffer = combinedBuffer;
  let processed = 0;
  remainingData = new Uint8Array(0);

  while (processed < buffer.length) {
    // 查找匹配的协议
    const protocol = Object.values(PROTOCOL_CONFIG).find(p =>
      buffer[processed] === p.responseHeader
    );

    if (!protocol) {
      processed++;
      continue;
    }

    const footerIndex = buffer.findIndex((byte, index) => {
      // 协议返回的是字符串
      if (protocol.frameLength === -1) {
        return index > processed && 
            byte === protocol.responseFooter;
      }
      else{
        // 长度验证，确保找到的是真正的协议尾
        return index > processed && 
              byte === protocol.responseFooter &&
              (index - processed + 1) === protocol.frameLength;
      }
    });

    if (footerIndex === -1) {
      remainingData = buffer.slice(processed);
      break;
    }

    const frame = buffer.slice(processed, footerIndex + 1);
    processed = footerIndex + 1;

    try {
      const result = protocol.handler(frame);

      if (protocol === PROTOCOL_CONFIG.WHO_AM_I) {
        document.getElementById("device_name").textContent = "Name: " + result;
      }
      else if (protocol === PROTOCOL_CONFIG.READ_SYSTEM_VERSION) {
        document.getElementById("device_version").textContent = "Version: " + result;
      }
      else if (protocol === PROTOCOL_CONFIG.READ_SYSTEM_SERIAL_NUM) {
        document.getElementById("device_serial").textContent = "SN: " + result.toUpperCase();
        if (result.toUpperCase().startsWith(DEVICE_PDPowerMiniV1)) {
          device_type = DEVICE_PDPowerMiniV1;
          if(versionGe(document.getElementById("device_version").textContent.split(" ")[1], "1.0.2.0")){
            showInputState.value = true;
          }
        }
        else if (result.toUpperCase().startsWith(DEVICE_PDPowerMiniV11)) {
          device_type = DEVICE_PDPowerMiniV11;
          if(versionGe(document.getElementById("device_version").textContent.split(" ")[1], "1.0.1.0")){
            showInputState.value = true;
          }
        }
        if(showInputState.value){
          setTimeout(() => {
            Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_INPUT_STATE.command));
          }, COMMAND_DELAY * 11);
        }
      }
      else if (protocol === PROTOCOL_CONFIG.READ_OUTPUT_EN_STATE) {
        setOutputEnabled(result.state & 0x01 ? true : false, true);
        measurements.value.mode = (result.state >> 1) === 0x00 ? 'OK' : (result.state >> 1) === 0x01 ? 'CC' : 'OC';
      }
      else if (protocol === PROTOCOL_CONFIG.READ_OUTPUT_ID) {
        switchGroup(result.id, true);
      }
      else if (protocol === PROTOCOL_CONFIG.READ_OUTPUT_DATA) {
        // 更新对应组的设置值
        settingsGroups.value[result.id] = {
          voltage: result.voltage,
          current: result.current
        };
        // 如果当前显示的是该组，更新显示
        if (currentGroup.value === result.id) {
          updateSettingsDisplay();
        }
      }
      else if (protocol === PROTOCOL_CONFIG.READ_OUTPUT_DISPLAY) {
        updateMeasurements(result.voltage, result.current);
      }
      else if (protocol === PROTOCOL_CONFIG.READ_INPUT_STATE) {
        document.getElementById("device_input_state").textContent = "Input Type: " + (INPUT_TYPE[result.state] || INPUT_TYPE[0]).text;
        document.getElementById("device_input_voltage").textContent = "Input Voltage: " + result.voltage + "V";
        if(pd_init == false){
          pd_init = true;
          pd_voltage.value = result.pd_voltage;
        }
        if(result.state != 0x05){
          showPDSettings.value = false;
        }
        else{
          showPDSettings.value = true;
        }
      }
    } catch (error) {
      console.error("协议处理错误:", error);
    }
  }
}

const COMMAND_DELAY = 50; // 命令发送间隔时间(ms)

async function Serial_write(data) {
  if (!isOpen) {
    message(t("not_open"), {
      title: t("warning"),
      kind: 'warning'
    });
    return;
  }
  const dataArray = Array.from(data);
  await invoke("serial_write", { data: dataArray });
}

async function DeviceInfoGet() {
  if (isOpen === false) {
    message(t("not_open"), {
      title: t("warning"),
      kind: 'warning'
    });
    return;
  }

  // 使用延时发送命令
  Serial_write(new Uint8Array(PROTOCOL_CONFIG.WHO_AM_I.command));

  setTimeout(() => {
    Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_SYSTEM_VERSION.command));
  }, COMMAND_DELAY * 1);

  setTimeout(() => {
    Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_SYSTEM_SERIAL_NUM.command));
  }, COMMAND_DELAY * 2);

  setTimeout(() => {
    Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_OUTPUT_EN_STATE.command));
  }, COMMAND_DELAY * 3);

  // 获取所有0-4组的输出数据，每组间隔COMMAND_DELAY
  for (let i = 0; i <= 4; i++) {
    setTimeout(() => {
      Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_OUTPUT_DATA.command(i)));
    }, COMMAND_DELAY * (4 + i));
  }

  setTimeout(() => {
    Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_OUTPUT_DISPLAY.command));
  }, COMMAND_DELAY * 9);

  setTimeout(() => {
    Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_OUTPUT_ID.command));
  }, COMMAND_DELAY * 10);

  setTimeout(() => {
    startPeriodicRead();
  }, COMMAND_DELAY * 12);
}

let readDisplayInterval;
let readEnableInterval;
let readInputInterval;
const READ_DISPLAY_INTERVAL = 200; // 200ms读取一次
const READ_ENABLE_INTERVAL = 500; // 500ms读取一次
const READ_INPUT_INTERVAL = 1000; // 2000ms读取一次

// 添加周期读取函数
function startPeriodicRead() {
  if (readDisplayInterval) {
    clearTimeout(readDisplayInterval);
  }

  const readDisplay = () => {
    if (isOpen) {
      Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_OUTPUT_DISPLAY.command));
    }
    readDisplayInterval = setTimeout(readDisplay, READ_DISPLAY_INTERVAL);
  };

  readDisplay();

  if (readEnableInterval) {
    clearTimeout(readEnableInterval);
  }

  const readEnable = () => {
    if (isOpen) {
      Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_OUTPUT_EN_STATE.command));
    }
    readEnableInterval = setTimeout(readEnable, READ_ENABLE_INTERVAL);
  };

  readEnable();

  const readInput = () => {
    if (isOpen && showInputState.value) {
      Serial_write(new Uint8Array(PROTOCOL_CONFIG.READ_INPUT_STATE.command));
    }
    readInputInterval = setTimeout(readInput, READ_INPUT_INTERVAL);
  };

  readInput();
}

function stopPeriodicRead() {
  if (readDisplayInterval) {
    clearTimeout(readDisplayInterval);
    readDisplayInterval = null;
  }

  if (readEnableInterval) {
    clearTimeout(readEnableInterval);
    readEnableInterval = null;
  }

  if (readInputInterval) {
    clearTimeout(readInputInterval);
    readInputInterval = null;
  }
}

function writeOutputEnable(enable) {
  if (!isOpen) {
    message(t("not_open"), {
      title: t("warning"),
      kind: 'warning'
    });
    return;
  }
  Serial_write(new Uint8Array(PROTOCOL_CONFIG.WRITE_OUTPUT_ENABLE.command(enable)));
}

function writeOutputID(id) {
  if (!isOpen) {
    message(t("not_open"), {
      title: t("warning"),
      kind: 'warning'
    });
    return;
  }
  Serial_write(new Uint8Array(PROTOCOL_CONFIG.WRITE_OUTPUT_ID.command(id)));
}

// 设备信息显示
const measurements = ref({
  voltage: 0,
  current: 0,
  mode: 'OK'
});

let chartDom;
let voltageCurrentChart;

onMounted(async () => {
  chartDom = document.getElementById('measurement-chart_');

  document.querySelectorAll('.settings-value.editable').forEach(el => {
    el.addEventListener('click', () => {
      const type = el.dataset.type;
      editSetting(type);
    });
  });

  // 初始化图表
  await initChart();

  updateSettingsDisplay();
});

// 根据主题获取图表配置
function getChartOption(isDarkMode) {
  return {
    tooltip: {
      trigger: 'axis',
      textStyle: {
        textAlign: 'left' // 设置 tooltip 文字左对齐
      },
    },
    legend: {
      data: [t('voltage') + '(V)', t('current') + '(A)'],
      textStyle: {
        color: isDarkMode ? '#ffffff' : '#000000' // 图例文字颜色
      }
    },
    grid: {
      top: '20%',
      bottom: '10%',
      left: '15%',
      right: '15%' // 增加右侧边距，给右侧 Y 轴留出空间
    },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: [],
      axisLine: {
        lineStyle: {
          color: isDarkMode ? '#ffffff' : '#000000' // x 轴颜色
        }
      },
      axisLabel: {
        color: isDarkMode ? '#ffffff' : '#000000' // x 轴标签颜色
      }
    },
    yAxis: [
      {
        type: 'value',
        name: t('voltage') + '(V)',
        position: 'left',
        axisLine: {
          lineStyle: {
            color: isDarkMode ? '#66b3ff' : '#4ec9b0' // 电压 Y 轴颜色
          }
        },
        axisLabel: {
          color: isDarkMode ? '#66b3ff' : '#4ec9b0' // 电压 Y 轴标签颜色
        }
      },
      {
        type: 'value',
        name: t('current') + '(A)',
        position: 'right',
        axisLine: {
          lineStyle: {
            color: isDarkMode ? '#ff6666' : '#f47d7d' // 电流 Y 轴颜色
          }
        },
        axisLabel: {
          color: isDarkMode ? '#ff6666' : '#f47d7d' // 电流 Y 轴标签颜色
        }
      }
    ],
    series: [
      {
        name: t('voltage') + '(V)',
        type: 'line',
        smooth: true,
        data: [],
        yAxisIndex: 0, // 指定使用左侧 Y 轴
        lineStyle: {
          color: isDarkMode ? '#66b3ff' : '#4ec9b0' // 电压曲线颜色
        }
      },
      {
        name: t('current') + '(A)',
        type: 'line',
        smooth: true,
        data: [],
        yAxisIndex: 1, // 指定使用右侧 Y 轴
        lineStyle: {
          color: isDarkMode ? '#ff6666' : '#f47d7d' // 电流曲线颜色
        }
      }
    ]
  };
}


// 初始化图表
async function initChart() {
  const isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
  voltageCurrentChart = echarts.init(chartDom);
  const option = getChartOption(isDarkMode);
  voltageCurrentChart.setOption(option);

  // 监听系统主题变化
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaQuery.addEventListener('change', (e) => {
    const newOption = getChartOption(e.matches);
    voltageCurrentChart.setOption(newOption);
  });
}

// 更新测量值函数
function updateMeasurements(voltage, current) {

  measurements.value.voltage = voltage;
  measurements.value.current = current;

  if (voltageCurrentChart) {
    const now = new Date();
    const timeLabel = [
      now.getHours().toString().padStart(2, '0'),
      now.getMinutes().toString().padStart(2, '0'),
      now.getSeconds().toString().padStart(2, '0')
    ].join(':');
    const option = voltageCurrentChart.getOption();

    // 添加新数据点
    option.xAxis[0].data.push(timeLabel);
    option.series[0].data.push(voltage);
    option.series[1].data.push(current);

    // 限制显示的数据点数
    if (option.xAxis[0].data.length > 20) {
      option.xAxis[0].data.shift();
      option.series[0].data.shift();
      option.series[1].data.shift();
    }

    // 使用notMerge: true提高性能
    voltageCurrentChart.setOption(option, {
      notMerge: false,
      lazyUpdate: true
    });
  }
}

// 设置组数据存储，使用 ref 使其响应式
const settingsGroups = ref({
  0: { voltage: 0, current: 0 },
  1: { voltage: 0, current: 0 },
  2: { voltage: 0, current: 0 },
  3: { voltage: 0, current: 0 },
  4: { voltage: 0, current: 0 }
});

// 当前选中的设置组，使用 ref 使其响应式
const currentGroup = ref(0);

// 设置组切换函数
const switchGroup = async (group, isInitial = false) => {
  // 如果使能输出，不允许切换组
  if (outputEnabled.value && !isInitial) {
    await message(t('outputEnabled'), {
      title: t('error'),
      kind: 'error'
    });
    return;
  }
  console.log('切换到设置组:', group);
  currentGroup.value = group;
  updateSettingsDisplay();

  // 如果不是初始化，则发送写入命令
  if (!isInitial && isOpen) {
    writeOutputID(group);
  }
};
// 更新设置显示函数
const updateSettingsDisplay = () => {
  const group = settingsGroups.value[currentGroup.value];
  document.querySelectorAll('.settings-value[data-type="voltage"]').forEach(el => {
    el.textContent = `${group.voltage.toFixed(3)} V`;
  });
  document.querySelectorAll('.settings-value[data-type="current"]').forEach(el => {
    el.textContent = `${group.current.toFixed(3)} A`;
  });
};

const showModal = ref(false);
const modalTitle = ref('');
const modalLabel = ref('');
const modalType = ref('');
const modalValue = ref('');
const modalMin = ref(null);
const modalMax = ref(null);
const modalCallback = ref(null);

// 自定义提示框
async function customPrompt(title, options) {
  return new Promise((resolve) => {
    modalTitle.value = title;
    modalLabel.value = options.title || '';
    modalType.value = options.kind || 'info';
    modalValue.value = options.defaultValue || '';
    modalMin.value = options.min;
    modalMax.value = options.max;
    modalCallback.value = resolve;
    showModal.value = true;
  });
}

// 替换原来的 editSetting 函数中的 prompt 调用
const UNIT_MAP = {
  voltage: 'V',
  current: 'A'
}

const editSetting = async (type) => {
  const groupKey = currentGroup.value
  const groupData = settingsGroups.value[groupKey]
  const unit = UNIT_MAP[type]
  const label = `${t(type)} (${unit})`
  const { min, max} = DEVICE_RANGE_CONFIG[device_type][type]
  const originVal = groupData[type]

  try {
    const inputStr = await customPrompt(t('edit_setting'), {
      title: `${t('edit')} ${label}`,
      kind: 'info',
      defaultValue: originVal.toString(),
      min,
      max
    })

    // 用户取消输入，直接退出
    if (inputStr === null) return

    const numValue = parseFloat(inputStr)
    // 非法数字校验
    if (Number.isNaN(numValue)) {
      await message(t('invalid_number'), { title: t('error'), kind: 'error' })
      return
    }

    // 区间越界校验
    if (numValue < min || numValue > max) {
      const errMsg = t('value_out_of_range')
        .replace('{min}', min)
        .replace('{max}', max)
      await message(errMsg, { title: t('error'), kind: 'error' })
      return
    }

    // 更新内存配置
    groupData[type] = numValue

    // 统一更新页面DOM显示
    const displayText = `${numValue.toFixed(3)} ${unit}`
    document.querySelectorAll(`.settings-value[data-type="${type}"]`)
      .forEach(el => el.textContent = displayText)

    // 串口已打开则下发配置指令
    if (isOpen) {
      const voltMv = Math.round(groupData.voltage * 1000)
      const currMv = Math.round(groupData.current * 1000)
      console.log('发送写入命令', groupKey, voltMv, currMv)
      const cmd = PROTOCOL_CONFIG.WRITE_OUTPUT_DATA.command(groupKey, voltMv, currMv)
      Serial_write(cmd)
    }
  } catch (error) {
    console.error('编辑设置出错:', error)
    await message(t('edit_failed'), { title: t('error'), kind: 'error' })
  }
}

const outputEnabled = ref(false);

function setOutputEnabled(enabled, isInitiated = false) {
  if (!isOpen)
    return;
  if (isInitiated) {
    outputEnabled.value = enabled;
  }
  else {
    writeOutputEnable(enabled);
  }
}

function toggleOutput() {
  if (isOpen) {
    if (outputEnabled.value === true) {

      setOutputEnabled(true);
    } else {
      setOutputEnabled(false);
    }
  }
  else {
    outputEnabled.value = false;
    message(t("not_open"), {
      title: t("warning"),
      kind: 'warning'
    });
    return;

  }
}

async function setting_save() {
  if (isOpen) {
    await Serial_write(PROTOCOL_CONFIG.WRITE_SYSTEM_CONFIG_SAVE.command);
  }
}

const showInputState = ref(false)
const showPDSettings = ref(false)
const pd_voltage = ref(8)
function handleBlurPDVoltage() {
  const { min, max} = DEVICE_RANGE_CONFIG[device_type].pd_voltage
  const val = pd_voltage.value

  // 边界钳位写法，更简洁
  pd_voltage.value = Math.max(min, Math.min(val, max))
}

async function pd_voltage_set() {
  if (isOpen) {
    if(outputEnabled.value === false && measurements.value.voltage < 5){
      const cmd = PROTOCOL_CONFIG.WRITE_INPUT_PD_VOLTAGE.command(pd_voltage.value*10)
      Serial_write(cmd)
    }
    else{
      message(t("pd_voltage_error"), {
        title: t("error"),
        kind: 'error'
      });
    }
  }
}

async function setting_refresh() {
  await DeviceInfoGet();
}

// 禁止右键和检查
//禁止F12
document.onkeydown = function (event: any) {
    var winEvent: any = window.event
    if (winEvent && winEvent.keyCode == 123) {
        event.keyCode = 0
        event.returnValue = false
    }
    // if (winEvent && winEvent.keyCode == 13) {
    //     winEvent.keyCode = 505
    // }
}
 
//屏蔽右键菜单
document.oncontextmenu = function (event: any) {
    if (window.event) {
        event = window.event
    }
    try {
        var the = event.srcElement
        if (
            !(
                (the.tagName == 'INPUT' && the.type.toLowerCase() == 'text') ||
                the.tagName == 'TEXTAREA'
            )
        ) {
            return false
        }
        return true
    } catch (e) {
        return false
    }
}

</script>

<template src="./App.html"></template>
<style src="./styles.css" lang="css"></style>