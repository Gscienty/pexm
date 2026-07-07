import { graphqlRequest } from './api';

type PaperTemplate = {
  id: number;
  name: string;
  description: string;
};

export async function initTemplateSettings(paperToken: string) {
  if (typeof window === 'undefined') return;

  const container = document.getElementById('templateSettingsContent');
  if (!container) return;

  const renderLoading = () => {
    container.innerHTML = `
      <div class="doc-settings-card">
        <strong>模板状态</strong>
        <span>加载中...</span>
      </div>
    `;
  };

  renderLoading();

  try {
    let hasTemplate = false;
    let currentTemplateName = '';
    let currentTemplateId: number | undefined;

    if (paperToken && paperToken !== 'new') {
      const data = await graphqlRequest(
        `query($paperId: Int!) { paperBreadcrumb(paperId: $paperId) { paper { id templateId } } }`,
        { paperId: parseInt(paperToken, 10) },
      );
      const templateId = data?.paperBreadcrumb?.paper?.templateId;
      currentTemplateId = templateId ? Number(templateId) : undefined;
      hasTemplate = Boolean(currentTemplateId && currentTemplateId > 0);
    }

    const templatesData = await graphqlRequest(`query { paperTemplates { id name description } }`);
    const templates: PaperTemplate[] = templatesData?.paperTemplates || [];

    if (!templates.length) {
      container.innerHTML = `
        <div class="doc-settings-card">
          <strong>模板状态</strong>
          <span>当前暂无可用模板。</span>
        </div>
      `;
      return;
    }

    if (hasTemplate && templates.length && currentTemplateId) {
      const currentTemplate = templates.find((template) => template.id === currentTemplateId);
      currentTemplateName = currentTemplate?.name || '';
    }

    const renderPicker = (selectedTemplateId?: number) => {
      container.innerHTML = `
        <div class="doc-settings-card doc-settings-card-stack">
          <div class="doc-settings-card-header">
            <strong>${hasTemplate ? '已选择模板' : '选择模板'}</strong>
            <span>${hasTemplate && currentTemplateName ? currentTemplateName : '你可以选择合适的试卷模板'}</span>
          </div>
          <button class="doc-settings-action-btn" type="button" id="templateSettingsTrigger">${hasTemplate ? '更换模板' : '选择模板'}</button>
          <div class="doc-settings-template-list" id="templateSettingsList" hidden></div>
        </div>
      `;

      const trigger = document.getElementById('templateSettingsTrigger');
      const list = document.getElementById('templateSettingsList');
      if (!trigger || !list) return;

      const toggleList = async () => {
        if (list.hidden) {
          list.hidden = false;
          list.innerHTML = templates.map((template) => `
            <button class="doc-settings-template-item ${selectedTemplateId === template.id ? 'active' : ''}" type="button" data-template-id="${template.id}">
              <span class="doc-settings-template-name">${template.name}</span>
              <span class="doc-settings-template-desc">${template.description || '暂无描述'}</span>
            </button>
          `).join('');

          list.querySelectorAll<HTMLButtonElement>('[data-template-id]').forEach((item) => {
            item.addEventListener('click', async () => {
              const templateId = Number(item.dataset.templateId);
              if (!templateId || !paperToken || paperToken === 'new') return;

              try {
                await graphqlRequest(
                  `mutation($paperId: Int!, $templateId: Int!) { setPaperTemplate(paperId: $paperId, templateId: $templateId) }`,
                  { paperId: parseInt(paperToken, 10), templateId },
                );
                const selectedTemplate = templates.find((template) => template.id === templateId);
                if (selectedTemplate) {
                  currentTemplateName = selectedTemplate.name;
                  hasTemplate = true;
                  renderPicker(templateId);
                }
              } catch (error) {
                console.error('Failed to set paper template:', error);
                if (list) {
                  list.innerHTML = `
                    <div class="doc-settings-template-empty">保存模板选择失败，请稍后再试。</div>
                  `;
                }
              }
            });
          });
        } else {
          list.hidden = true;
          list.innerHTML = '';
        }
      };

      trigger.addEventListener('click', () => {
        void toggleList();
      });
    };

    renderPicker(currentTemplateId);
  } catch (error) {
    console.error('Failed to load template settings:', error);
    container.innerHTML = `
      <div class="doc-settings-card">
        <strong>模板状态</strong>
        <span>暂时无法读取模板配置，请稍后再试。</span>
      </div>
    `;
  }
}
