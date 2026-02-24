import type { JSX, ValidComponent } from "solid-js"
import { splitProps } from "solid-js"

import type { PolymorphicProps } from "@kobalte/core"
import * as CheckboxPrimitive from "@kobalte/core/checkbox"

import { cn } from "~/lib/utils"

const CheckboxRoot = CheckboxPrimitive.Root
const CheckboxDescription = CheckboxPrimitive.Description
const CheckboxErrorMessage = CheckboxPrimitive.ErrorMessage

type CheckboxControlProps = CheckboxPrimitive.CheckboxControlProps & {
  class?: string | undefined
  children?: JSX.Element
}

const CheckboxControl = <T extends ValidComponent = "input">(
  props: PolymorphicProps<T, CheckboxControlProps>
) => {
  const [local, others] = splitProps(props as CheckboxControlProps, ["class", "children"])
  return (
    <>
      <CheckboxPrimitive.Input
        class={cn(
          "[&:focus-visible+div]:outline-none [&:focus-visible+div]:ring-2 [&:focus-visible+div]:ring-ring [&:focus-visible+div]:ring-offset-2 [&:focus-visible+div]:ring-offset-background",
          local.class
        )}
      />
      <CheckboxPrimitive.Control
        class={cn(
          "h-4 w-4 shrink-0 rounded border border-primary ring-offset-background data-[checked]:bg-primary data-[checked]:text-primary-foreground data-[disabled]:cursor-not-allowed data-[disabled]:opacity-50",
          local.class
        )}
        {...others}
      >
        {local.children || (
          <CheckboxPrimitive.Indicator class="flex items-center justify-center text-current">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </CheckboxPrimitive.Indicator>
        )}
      </CheckboxPrimitive.Control>
    </>
  )
}

type CheckboxLabelProps = CheckboxPrimitive.CheckboxLabelProps & { class?: string | undefined }

const CheckboxLabel = <T extends ValidComponent = "label">(
  props: PolymorphicProps<T, CheckboxLabelProps>
) => {
  const [local, others] = splitProps(props as CheckboxLabelProps, ["class"])
  return (
    <CheckboxPrimitive.Label
      class={cn(
        "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
        local.class
      )}
      {...others}
    />
  )
}

// Backward compatibility wrapper
type CheckboxProps<T extends ValidComponent = "input"> = CheckboxPrimitive.CheckboxRootProps<T> & {
  class?: string | undefined
  label?: string
}

const Checkbox = <T extends ValidComponent = "input">(
  props: PolymorphicProps<T, CheckboxProps<T>>
) => {
  const [local, others] = splitProps(props as CheckboxProps, ["class", "label"])
  return (
    <CheckboxRoot class={cn("flex items-center gap-2", local.class)} {...others}>
      <CheckboxControl />
      {local.label && <CheckboxLabel>{local.label}</CheckboxLabel>}
    </CheckboxRoot>
  )
}

export { Checkbox, CheckboxRoot, CheckboxControl, CheckboxLabel, CheckboxDescription, CheckboxErrorMessage }
export type { CheckboxProps }
