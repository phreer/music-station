const API_BASE = window.location.origin

class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

interface RequestOptions extends Omit<RequestInit, 'signal'> {
  signal?: AbortSignal
  /** Max network-error retries (default 2). HTTP 4xx/5xx are NOT retried. */
  retries?: number
}

async function request<T>(path: string, options?: RequestOptions): Promise<T> {
  const url = `${API_BASE}${path}`
  const { retries = 2, ...fetchOpts } = options ?? {}

  let lastError: unknown
  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const response = await fetch(url, fetchOpts)

      if (!response.ok) {
        const text = await response.text().catch(() => 'Unknown error')
        throw new ApiError(response.status, `${response.status}: ${text}`)
      }

      const contentType = response.headers.get('content-type')
      if (contentType?.includes('application/json')) {
        return response.json() as Promise<T>
      }

      return response.text() as unknown as T
    } catch (e) {
      // Never retry aborted requests or HTTP errors (4xx/5xx)
      if (e instanceof ApiError || (e instanceof DOMException && e.name === 'AbortError')) {
        throw e
      }
      lastError = e
      if (attempt < retries) {
        // Exponential back-off: 500ms, 1000ms, ...
        await new Promise((r) => setTimeout(r, 500 * 2 ** attempt))
      }
    }
  }
  throw lastError
}

export async function get<T>(path: string, signal?: AbortSignal): Promise<T> {
  return request<T>(path, { signal })
}

export async function post<T>(path: string, body?: unknown, signal?: AbortSignal): Promise<T> {
  return request<T>(path, {
    method: 'POST',
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
    signal,
  })
}

export async function put<T>(path: string, body?: unknown, signal?: AbortSignal): Promise<T> {
  return request<T>(path, {
    method: 'PUT',
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
    signal,
  })
}

export async function putFormData<T>(path: string, formData: FormData, signal?: AbortSignal): Promise<T> {
  return request<T>(path, {
    method: 'PUT',
    body: formData,
    signal,
  })
}

export async function postFormData<T>(path: string, formData: FormData, signal?: AbortSignal): Promise<T> {
  return request<T>(path, {
    method: 'POST',
    body: formData,
    signal,
  })
}

export async function del<T = void>(path: string, signal?: AbortSignal): Promise<T> {
  return request<T>(path, { method: 'DELETE', signal })
}

export function streamUrl(trackId: string): string {
  return `${API_BASE}/stream/${trackId}`
}

export function coverUrl(trackId: string): string {
  return `${API_BASE}/cover/${trackId}`
}

export { ApiError }
