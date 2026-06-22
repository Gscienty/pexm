const BFF_BASE = '';

export async function graphqlRequest(query: string, variables: Record<string, unknown> = {}): Promise<any> {
  const res = await fetch(`${BFF_BASE}/api/graphql`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ query, variables }),
  });
  if (!res.ok) throw new Error('网络错误');
  const json = await res.json();
  if (json.errors) throw new Error(json.errors[0]?.message || 'GraphQL 错误');
  return json.data;
}

export async function checkAuth() {
  try {
    const res = await fetch(`${BFF_BASE}/api/auth/me`, { credentials: 'include' });
    if (!res.ok) { window.location.href = '/login'; return null; }
    return res.json();
  } catch {
    window.location.href = '/login';
    return null;
  }
}

export function escapeHtml(str: string): string {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
