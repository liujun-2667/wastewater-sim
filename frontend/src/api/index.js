import axios from 'axios'

const api = axios.create({
  baseURL: '/api',
  timeout: 60000
})

api.interceptors.response.use(
  response => response.data,
  error => {
    console.error('API Error:', error)
    return Promise.reject(error)
  }
)

export const simulationApi = {
  simulate(data) {
    return api.post('/simulate', data)
  },

  getTemplates() {
    return api.get('/templates')
  },

  sensitivity(data) {
    return api.post('/sensitivity', data)
  },

  sensitivity2d(data) {
    return api.post('/sensitivity/2d', data)
  },

  parseCsv(csvContent) {
    return api.post('/csv/parse', { csv_content: csvContent })
  }
}

export default api
