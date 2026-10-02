import { memo, useCallback, useEffect, useRef, useState } from 'react'

export interface Flags {
  timestamps: boolean
  ids: boolean
  replies: boolean
  edited: boolean
  merge: boolean
}

interface FlagsSelectorProps {
  value: Flags
  onChange: (flags: Flags) => void
  disabled?: boolean
}

const flagsList: { key: keyof Flags; label: string }[] = [
  { key: 'timestamps', label: 'Timestamps' },
  { key: 'ids', label: 'Message IDs' },
  { key: 'replies', label: 'Replies' },
  { key: 'edited', label: 'Edited time' },
  { key: 'merge', label: 'Merge consecutive' },
]

export const FlagsSelector = memo(function FlagsSelector({
  value,
  onChange,
  disabled = false,
}: FlagsSelectorProps) {
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const handleToggle = useCallback(
    (flag: keyof Flags) => {
      const next = {
        ...value,
        [flag]: !value[flag],
      }
      if (flag === 'replies' && next.replies) {
        next.ids = true
      }
      if (next.ids) next.merge = false
      onChange(next)
    },
    [onChange, value],
  )

  const selectedCount = flagsList.filter(({ key }) => value[key]).length
  const label = selectedCount === 0 ? 'No flags' : `${selectedCount} selected`

  return (
    <div style={styles.container} ref={containerRef}>
      <button
        type="button"
        onClick={() => !disabled && setIsOpen(!isOpen)}
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            setIsOpen(false)
          }
        }}
        disabled={disabled}
        aria-expanded={isOpen}
        aria-haspopup="menu"
        style={{
          ...styles.trigger,
          ...(isOpen ? styles.triggerOpen : {}),
          ...(disabled ? styles.triggerDisabled : {}),
        }}
      >
        <span style={styles.value}>{label}</span>
        <svg
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          style={{
            transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)',
            transition: 'transform 150ms ease',
          }}
          aria-hidden="true"
        >
          <path
            d="M2.5 4.5L6 8L9.5 4.5"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>

      {isOpen && (
        <div style={styles.dropdown} role="menu" aria-label="Output flags">
          {flagsList.map(({ key, label: flagLabel }) => (
            <label key={key} style={styles.option}>
              <input
                type="checkbox"
                checked={value[key]}
                onChange={() => handleToggle(key)}
                disabled={
                  disabled || (value.replies && key === 'ids') || (value.ids && key === 'merge')
                }
                style={styles.input}
              />
              <span style={styles.optionLabel}>{flagLabel}</span>
            </label>
          ))}
          {value.replies && (
            <span style={styles.help}>
              Replies include IDs; merging is off to keep targets exact.
            </span>
          )}
          {value.ids && !value.replies && (
            <span style={styles.help}>Message IDs turn off merging to keep each ID exact.</span>
          )}
        </div>
      )}
    </div>
  )
})

const styles: Record<string, React.CSSProperties> = {
  container: {
    position: 'relative',
    width: '100%',
  },
  trigger: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    width: '100%',
    padding: '12px 16px',
    fontFamily: 'var(--font-mono)',
    fontSize: '14px',
    color: 'var(--text-primary)',
    background: 'var(--bg-tertiary)',
    border: '1px solid var(--border-default)',
    borderRadius: 'var(--radius-md)',
    cursor: 'pointer',
    transition: 'border-color var(--transition-fast), background var(--transition-fast)',
  },
  triggerOpen: {
    borderColor: 'var(--accent-green)',
  },
  triggerDisabled: {
    opacity: 0.5,
    cursor: 'not-allowed',
  },
  value: {
    flex: 1,
    textAlign: 'left',
  },
  dropdown: {
    position: 'absolute',
    top: 'calc(100% + 4px)',
    left: 0,
    right: 0,
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    padding: '8px',
    background: 'var(--bg-elevated)',
    border: '1px solid var(--border-default)',
    borderRadius: 'var(--radius-md)',
    zIndex: 100,
    boxShadow: '0 8px 24px rgba(0, 0, 0, 0.4)',
  },
  option: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    minHeight: '40px',
    padding: '8px',
    borderRadius: 'var(--radius-sm)',
    cursor: 'pointer',
    userSelect: 'none',
  },
  input: {
    width: '20px',
    height: '20px',
    margin: 0,
    accentColor: 'var(--accent-green)',
    cursor: 'pointer',
    flexShrink: 0,
  },
  optionLabel: {
    fontFamily: 'var(--font-mono)',
    fontSize: '13px',
    color: 'var(--text-primary)',
    lineHeight: 1.3,
  },
  help: {
    padding: '4px 8px',
    color: 'var(--text-secondary)',
    fontSize: '12px',
    lineHeight: 1.4,
  },
}
