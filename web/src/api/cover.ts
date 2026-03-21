import { postFormData, del } from './client'

export async function uploadCover(trackId: string, file: File): Promise<void> {
  const formData = new FormData()
  formData.append('cover', file)
  await postFormData<void>(`/cover/${trackId}`, formData)
}

export async function deleteCover(trackId: string): Promise<void> {
  await del(`/cover/${trackId}`)
}
