// 紧凑数字输入格。

export function NumField({ label, v, on, w = 58 }: {
  label: string; v: number; on: (e: { target: { value: string } }) => void; w?: number
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input type="number" value={v} onChange={on} style={{ width: w }} />
    </label>
  )
}
