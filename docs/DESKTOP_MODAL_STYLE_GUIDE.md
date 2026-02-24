# Desktop-Native Modal & Form Style Guide

This guide documents the VS Code-inspired desktop styling patterns applied to the Lines page modals. Follow these rules to migrate **all remaining modals and forms** across the app to the same desktop-native look.

## Reference Implementation

The completed Lines page modals are the canonical reference:

- `src/frontend/src/pages/lines/_components/modals/add-line-dialog.tsx`
- `src/frontend/src/pages/lines/_components/modals/view-line-dialog.tsx`
- `src/frontend/src/pages/lines/_components/modals/delete-line-dialog.tsx`
- `src/frontend/src/pages/lines/_components/modals/sections/BasicInfoSection.tsx`
- `src/frontend/src/pages/lines/_components/modals/sections/NetworkSection.tsx`
- `src/frontend/src/pages/lines/_components/modals/sections/CredentialsSection.tsx`

The base dialog component has already been restyled:

- `src/frontend/src/components/ui/dialog.tsx` (already done - do NOT modify)

---

## Files That Need Migration

### Task Modals
- `src/frontend/src/pages/tasks/_components/modals/EditTaskModal.tsx`
- `src/frontend/src/pages/tasks/_components/modals/ExecutionHistoryModal.tsx`
- `src/frontend/src/pages/tasks/_components/modals/RunNowDialog.tsx`
- `src/frontend/src/pages/tasks/_components/wizard/CreateTaskWizard.tsx`
- `src/frontend/src/pages/tasks/_components/wizard/steps/StepTaskName.tsx`
- `src/frontend/src/pages/tasks/_components/wizard/steps/StepNotifications.tsx`

### Settings / Email Dialogs
- `src/frontend/src/components/settings/email/EmailRecipientFormDialog.tsx`
- `src/frontend/src/components/settings/email/EmailRecipientDeleteDialog.tsx`
- `src/frontend/src/components/settings/email/SmtpFormDialog.tsx`
- `src/frontend/src/components/settings/email/SmtpDeleteDialog.tsx`
- `src/frontend/src/components/settings/email/SmtpTestDialog.tsx`

---

## Color Palette

| Token              | Value       | Usage                              |
|--------------------|-------------|-------------------------------------|
| Panel background   | `#252526`   | Dialog content background           |
| Header/Footer bg   | `#2d2d2d`   | Title bar, action bar               |
| Input background   | `#1e1e1e`   | Text inputs, selects                |
| Nested input bg    | `#2d2d2d`   | Inputs inside dark panels           |
| Border             | `#3c3c3c`   | All borders (dialog, inputs, dividers) |
| Subtle border      | `#2a2a2a`   | Row separators, nested panels       |
| Text primary       | `#cccccc`   | Body text, input values             |
| Text muted         | `#808080`   | Labels, secondary text              |
| Text disabled      | `#555`      | Placeholders, hints                 |
| Accent             | `#007acc`   | Focus borders, primary buttons, active states |
| Accent hover       | `#1a85c4`   | Primary button hover                |
| Destructive        | `#c72e0f`   | Delete buttons, error indicators    |
| Destructive hover  | `#d9534f`   | Delete button hover                 |
| Overlay            | `black/50`  | Dialog backdrop                     |

---

## Dialog Structure

Every dialog follows this structure:

```tsx
<Dialog open={...} onOpenChange={...}>
  <DialogContent class="max-w-[Xpx]">
    {/* Title bar */}
    <DialogHeader>
      <DialogTitle>Title Here</DialogTitle>
      <button
        type="button"
        class="text-[#808080] hover:text-[#cccccc] p-0.5 rounded-[2px] hover:bg-white/[0.06] transition-colors"
        onClick={() => onOpenChange(false)}
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/>
        </svg>
      </button>
    </DialogHeader>

    {/* Body */}
    <div class="px-4 py-3">
      {/* content */}
    </div>

    {/* Action bar */}
    <DialogFooter>
      {/* Cancel + Primary buttons */}
    </DialogFooter>
  </DialogContent>
</Dialog>
```

### Key Rules

1. **DialogHeader** is now a flex row (`justify-between`). Always put `<DialogTitle>` on the left and a custom close (X) SVG button on the right.
2. **Do NOT use the old `<DialogPrimitive.CloseButton>`** — it was removed from the base component. Each modal provides its own X button in the header.
3. **DialogContent** max-widths:
   - Form dialogs: `max-w-[520px]`
   - View/detail dialogs: `max-w-[460px]`
   - Confirm/delete dialogs: `max-w-[400px]`
   - Large/wizard dialogs: `max-w-[600px]`
4. **Body** uses `px-4 py-3` padding directly, NOT `p-6` or `gap-4`.
5. **Do NOT use shadcn `<Button>` component** inside modals. Use native `<button>` elements with the desktop classes below.

---

## Buttons

### Cancel / Secondary Button
```
h-[26px] px-3 text-[12px] rounded-[3px] border border-[#3c3c3c] bg-[#2d2d2d] text-[#cccccc]
hover:bg-[#3c3c3c] hover:border-[#808080] transition-colors
```

### Primary Button
```
h-[26px] px-3 text-[12px] rounded-[3px] border border-[#007acc] bg-[#007acc] text-white
hover:bg-[#1a85c4] hover:border-[#1a85c4] disabled:opacity-50 disabled:cursor-not-allowed transition-colors
```

### Destructive Button
```
h-[26px] px-3 text-[12px] rounded-[3px] border border-[#c72e0f] bg-[#c72e0f] text-white
hover:bg-[#d9534f] hover:border-[#d9534f] transition-colors
```

---

## Form Inputs

### Standard Input
```
h-[28px] w-full px-2 bg-[#1e1e1e] border border-[#3c3c3c] text-[#cccccc] text-[12px] rounded-none
focus:outline-none focus:border-[#007acc] placeholder:text-[#555]
```

- For "primary" fields (like Name): use `h-[34px]` instead of `h-[28px]`
- For technical/IP fields: add `font-mono`
- For inputs inside a dark nested panel: use `bg-[#2d2d2d]` instead of `bg-[#1e1e1e]`
- For selects: append `cursor-pointer` to the input class
- For password fields with toggle: add `pr-7` to leave room for the eye icon button

### Labels
```
text-[11px] text-[#808080] mb-1 block
```

Required indicators: `<span class="text-[#c72e0f]">*</span>`

### Section Headers
```
text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-2
```

### Hint Text
```
text-[10px] text-[#555] mt-1
```

---

## Form Sections

### Standard Section
```tsx
<div>
  <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-2">
    Section Name
  </div>
  <div class="grid grid-cols-2 gap-3">
    {/* fields */}
  </div>
</div>
```

### Nested/Secure Section (e.g., Credentials)
Wrap inputs in a dark panel:
```tsx
<div class="bg-[#1e1e1e] rounded-none border border-[#2a2a2a] p-3">
  <div class="grid grid-cols-2 gap-3">
    {/* inputs with bg-[#2d2d2d] instead of bg-[#1e1e1e] */}
  </div>
</div>
```

### Section Separators
Use thin lines between sections in the body:
```tsx
<div class="h-px bg-[#3c3c3c]" />
```

---

## View/Detail Dialogs

For read-only detail modals, group data into labeled panels:

### Detail Row
```tsx
<div class="flex items-baseline justify-between py-1.5 border-b border-[#2a2a2a] last:border-b-0">
  <span class="text-[11px] text-[#808080] uppercase tracking-wider">{label}</span>
  <span class="text-[12px] text-[#cccccc]">{value}</span>
</div>
```

### Detail Panel
```tsx
<div class="mb-3">
  <div class="text-[10px] text-[#808080] uppercase tracking-wider font-medium mb-1.5">
    Group Name
  </div>
  <div class="bg-[#1e1e1e] rounded-[3px] border border-[#2a2a2a] px-3">
    <DetailRow label="Field" value="value" />
    <DetailRow label="Field" value="value" mono />
  </div>
</div>
```

Use `font-mono` on the value span for technical data (IPs, dates, IDs).

---

## Delete/Confirm Dialogs

```tsx
<div class="px-4 py-4">
  <div class="flex items-start gap-3">
    {/* Warning icon in tinted container */}
    <div class="flex-shrink-0 w-8 h-8 rounded-[3px] bg-[#c72e0f]/15 flex items-center justify-center">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="#c72e0f">
        <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm-.7 3h1.4L8.4 9H7.6L7.3 5zm.7 5.5a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6z"/>
      </svg>
    </div>
    <div>
      <p class="text-[13px] text-[#cccccc] mb-1">
        Are you sure you want to delete <strong class="text-white">"item name"</strong>?
      </p>
      <p class="text-[11px] text-[#808080]">
        This action cannot be undone.
      </p>
    </div>
  </div>
</div>
```

---

## What to Replace

When migrating a modal, replace these patterns:

| Old (Web/shadcn)                                      | New (Desktop)                                      |
|-------------------------------------------------------|----------------------------------------------------|
| `<Button variant="outline">Cancel</Button>`           | Native `<button>` with cancel class (see above)    |
| `<Button>Save</Button>`                               | Native `<button>` with primary class (see above)   |
| `<Button variant="destructive">Delete</Button>`       | Native `<button>` with destructive class           |
| `<TextField>` + `<TextFieldInput>` + `<TextFieldLabel>` | Plain `<label>` + `<input>` with desktop classes |
| `<Select>` + `<SelectTrigger>` + `<SelectContent>`    | Plain `<select>` with input class + `cursor-pointer` |
| `<Separator />`                                       | `<div class="h-px bg-[#3c3c3c]" />`              |
| `<Badge variant="secondary">`                         | Inline `<span>` with appropriate color             |
| `class="space-y-3 py-3"` on body                      | `class="px-4 py-3 space-y-4"` with separators     |
| `class="text-sm font-medium"` on labels               | `text-[11px] text-[#808080] mb-1 block`           |
| `class="text-sm text-muted-foreground"` on hints      | `text-[10px] text-[#555]`                         |
| `class="rounded-lg border border-border bg-muted/30"` | `bg-[#1e1e1e] rounded-none border border-[#2a2a2a]` |
| Kobalte `<Switch.Root>` in modals                      | Keep as-is (already desktop-styled in table)       |

---

## Icons

Use inline SVGs instead of lucide-solid icon imports where possible. Common icons:

### Close (X)
```html
<svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
  <path d="M8 8.707l3.646 3.647.708-.708L8.707 8l3.647-3.646-.708-.708L8 7.293 4.354 3.646l-.708.708L7.293 8l-3.647 3.646.708.708L8 8.707z"/>
</svg>
```

### Edit (Pencil)
```html
<svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
  <path d="M13.23 1h-1.46L3.52 9.25l-.16.22L1 13.59 2.41 15l4.12-2.36.22-.16L15 4.23V2.77L13.23 1zM2.41 13.59l1.51-3 1.45 1.45-2.96 1.55zm3.83-2.06L4.47 9.76l6-6 1.77 1.77-6 6z"/>
</svg>
```

### Lock
```html
<svg width="11" height="11" viewBox="0 0 16 16" fill="#808080">
  <path d="M8 1a4 4 0 0 0-4 4v2H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V8a1 1 0 0 0-1-1h-1V5a4 4 0 0 0-4-4zm0 1a3 3 0 0 1 3 3v2H5V5a3 3 0 0 1 3-3zm-5 6h10v6H3V8z"/>
</svg>
```

### Warning (for delete dialogs)
```html
<svg width="16" height="16" viewBox="0 0 16 16" fill="#c72e0f">
  <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1a6 6 0 1 1 0 12A6 6 0 0 1 8 2zm-.7 3h1.4L8.4 9H7.6L7.3 5zm.7 5.5a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6z"/>
</svg>
```

---

## Wizard Dialogs

For multi-step wizards (like `CreateTaskWizard`):

- Use the same dialog shell (DialogHeader with title + X, DialogFooter with Back/Next buttons)
- Add a step indicator bar below the header:
  ```
  flex items-center gap-2 px-4 py-2 bg-[#1e1e1e] border-b border-[#3c3c3c]
  ```
- Step labels: `text-[11px]` with active step in `text-[#007acc]` and inactive in `text-[#555]`
- Step connector lines: `h-px flex-1 bg-[#3c3c3c]` (active: `bg-[#007acc]`)
- "Back" button uses the cancel/secondary style
- "Next" / "Create" button uses the primary style
- Body content per step follows the same form section patterns

---

## Checklist

For each modal file:

1. Remove imports of `Button` from `~/components/ui/button`
2. Remove imports of `TextField`, `TextFieldInput`, `TextFieldLabel` from `~/components/ui/text-field`
3. Remove imports of `Select`, `SelectTrigger`, `SelectValue`, `SelectContent`, `SelectItem`, `SelectLabel` from `~/components/ui/select`
4. Remove imports of `Separator` from `~/components/ui/separator`
5. Remove imports of `Badge` from `~/components/ui/badge` (if used inside modals)
6. Remove lucide-solid icon imports that can be replaced with inline SVGs
7. Replace all form elements with plain HTML + desktop classes
8. Add the close X button to DialogHeader
9. Replace DialogFooter contents with native buttons
10. Verify the dialog compiles: `npx tsc --noEmit`
