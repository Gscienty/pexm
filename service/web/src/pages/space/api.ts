const BFF_BASE = '';

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

export async function graphqlRequest(query: string, variables: Record<string, any> = {}) {
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

export async function loadSpace(token: string) {
	const data = await graphqlRequest(`query($token: String) { examSpaces(token: $token) { ... on GqlExamSpacePayload { space { id token name description lastUpdatedAt memberCount paperCount } } } }`, { token });
	return data?.examSpaces?.space || null;
}

export async function loadMembers(spaceId: number) {
	const data = await graphqlRequest(`query($examSpaceId: String!) { examSpaceMembers(examSpaceId: $examSpaceId) { id userId examSpaceId role username } }`, { examSpaceId: String(spaceId) });
	return data?.examSpaceMembers || [];
}

export async function loadDirectoryChildren(examSpaceId: number, parentId: number | null) {
	const data = await graphqlRequest(`query($examSpaceId: Int!, $parentId: Int) { directoryChildren(examSpaceId: $examSpaceId, parentId: $parentId) { directories { id examSpaceId parentId name createdAt updatedAt } papers { id examSpaceId directoryId title createdAt updatedAt } } }`, { examSpaceId, parentId });
	return data?.directoryChildren || { directories: [], papers: [] };
}

export async function loadDirectoryBreadcrumb(directoryId: number) {
	const data = await graphqlRequest(`query($directoryId: Int!) { directoryBreadcrumb(directoryId: $directoryId) { id name } }`, { directoryId });
	return data?.directoryBreadcrumb || [];
}

export async function createDirectory(examSpaceId: number, parentId: number | null, name: string) {
	const data = await graphqlRequest(`mutation($examSpaceId: Int!, $parentId: Int, $name: String!) { createDirectory(examSpaceId: $examSpaceId, parentId: $parentId, name: $name) { id name } }`, { examSpaceId, parentId, name });
	return data?.createDirectory;
}

export async function createPaper(examSpaceId: number, directoryId: number | null, title: string) {
	const data = await graphqlRequest(`mutation($examSpaceId: Int!, $directoryId: Int, $title: String!) { createPaper(examSpaceId: $examSpaceId, directoryId: $directoryId, title: $title) { id title } }`, { examSpaceId, directoryId, title });
	return data?.createPaper;
}

export async function deleteDirectory(id: number) {
	await graphqlRequest(`mutation($id: Int!) { deleteDirectory(id: $id) }`, { id });
}

export async function renameDirectory(id: number, name: string) {
	await graphqlRequest(`mutation($id: Int!, $name: String!) { updateDirectory(id: $id, name: $name) }`, { id, name });
}

export async function renamePaper(id: number, title: string) {
	await graphqlRequest(`mutation($id: Int!, $title: String!) { updatePaper(id: $id, title: $title) }`, { id, title });
}

export async function updateExamSpace(id: number, name: string, description: string) {
	await graphqlRequest(`mutation($id: Int!, $name: String!, $description: String!) { updateExamSpace(id: $id, name: $name, description: $description) }`, { id, name, description });
}

export async function deletePaper(id: number) {
	await graphqlRequest(`mutation($id: Int!) { deletePaper(id: $id) }`, { id });
}
