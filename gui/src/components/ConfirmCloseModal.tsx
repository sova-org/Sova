import React from 'react';
import { X } from 'lucide-react';

interface ConfirmCloseModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export const ConfirmCloseModal: React.FC<ConfirmCloseModalProps> = ({
  isOpen,
  onClose,
  onConfirm,
}) => {
  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{
        background: 'rgba(0, 0, 0, 0.4)',
        backdropFilter: 'blur(2px)'
      }}
    >
      <div
        className="w-full max-w-sm mx-4 shadow-2xl border"
        style={{
          backgroundColor: 'var(--color-surface)',
          borderColor: 'var(--color-border)',
          color: 'var(--color-text)'
        }}
      >
        <div
          className="flex justify-between items-center px-6 py-4 border-b"
          style={{ borderColor: 'var(--color-border)' }}
        >
          <h3
            className="text-lg font-semibold"
            style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}
          >
            Confirm Close
          </h3>
          <button
            onClick={onClose}
            className="transition-colors hover:opacity-70"
            style={{ color: 'var(--color-muted)' }}
          >
            <X size={20} />
          </button>
        </div>

        <div className="px-6 py-4">
          <p
            className="text-sm leading-relaxed"
            style={{ color: 'var(--color-text)' }}
          >
            Are you sure you want to close the application? This will stop the server (if started) and disconnect your client.
          </p>
        </div>

        <div
          className="flex justify-end space-x-3 px-6 py-4 border-t"
          style={{ borderColor: 'var(--color-border)' }}
        >
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium transition-colors border"
            style={{
              backgroundColor: 'var(--color-surface)',
              borderColor: 'var(--color-border)',
              color: 'var(--color-text)'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-primary-100)';
              e.currentTarget.style.color = 'var(--color-primary-700)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-surface)';
              e.currentTarget.style.color = 'var(--color-text)';
            }}
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 text-sm font-medium transition-colors border"
            style={{
              backgroundColor: 'var(--color-primary)',
              borderColor: 'var(--color-primary)',
              color: 'white'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-primary-600)';
              e.currentTarget.style.borderColor = 'var(--color-primary-600)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-primary)';
              e.currentTarget.style.borderColor = 'var(--color-primary)';
            }}
          >
            Close App
          </button>
        </div>
      </div>
    </div>
  );
};
