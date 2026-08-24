async function configureSidePanel() {
  await chrome.sidePanel.setPanelBehavior({ openPanelOnActionClick: true });
}

chrome.runtime.onInstalled.addListener(() => {
  configureSidePanel().catch((error) => {
    console.error('Focusa Workforce could not configure the side panel', error);
  });
});

chrome.runtime.onStartup.addListener(() => {
  configureSidePanel().catch((error) => {
    console.error('Focusa Workforce could not restore side-panel behavior', error);
  });
});
