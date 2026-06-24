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

/** Compile a paper to PDF via the BFF compile-pdf endpoint.
 *  Returns a Blob containing the PDF data. */
export async function compilePdf(paperToken: string): Promise<Blob> {
  const res = await fetch(`${BFF_BASE}/api/papers/${encodeURIComponent(paperToken)}/compile-pdf`, {
    method: 'POST',
    credentials: 'include',
  });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(text || `编译失败 (${res.status})`);
  }
  return res.blob();
}
