import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { FlagsSelector } from './FlagsSelector'

describe('FlagsSelector', () => {
  it('keeps reply targets available when replies are enabled', () => {
    const onChange = vi.fn()
    render(
      <FlagsSelector
        value={{ timestamps: false, ids: false, replies: false, edited: false, merge: true }}
        onChange={onChange}
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: /selected/i }))
    fireEvent.click(screen.getByRole('checkbox', { name: 'Replies' }))

    expect(onChange).toHaveBeenCalledWith({
      timestamps: false,
      ids: true,
      replies: true,
      edited: false,
      merge: false,
    })
  })
})
