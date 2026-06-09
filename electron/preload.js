const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('oplire', {
  getProxyStatus: () => ipcRenderer.invoke('get-proxy-status'),
  openExternal: (url) => ipcRenderer.send('open-external', url),
});
