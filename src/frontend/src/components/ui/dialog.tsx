import type { Component, ComponentProps, JSX, ValidComponent } from "solid-js"
import { splitProps } from "solid-js"

import * as DialogPrimitive from "@kobalte/core/dialog"
import type { PolymorphicProps } from "@kobalte/core/polymorphic"

import { cn } from "~/lib/utils"

const Dialog = DialogPrimitive.Root
const DialogTrigger = DialogPrimitive.Trigger

const DialogPortal: Component<DialogPrimitive.DialogPortalProps> = (props) => {
  const [, rest] = splitProps(props, ["children"])
  return (
    <DialogPrimitive.Portal {...rest}>
      <div class="fixed inset-0 z-50 flex items-center justify-center">
        {props.children}
      </div>
    </DialogPrimitive.Portal>
  )
}

type DialogOverlayProps<T extends ValidComponent = "div"> =
  DialogPrimitive.DialogOverlayProps<T> & { class?: string | undefined }

const DialogOverlay = <T extends ValidComponent = "div">(
  props: PolymorphicProps<T, DialogOverlayProps<T>>
) => {
  const [, rest] = splitProps(props as DialogOverlayProps, ["class"])
  return (
    <DialogPrimitive.Overlay
      class={cn(
        "fixed inset-0 z-50 bg-black/50 data-[expanded]:animate-in data-[closed]:animate-out data-[closed]:fade-out-0 data-[expanded]:fade-in-0",
        props.class
      )}
      {...rest}
    />
  )
}

type DialogContentProps<T extends ValidComponent = "div"> =
  DialogPrimitive.DialogContentProps<T> & {
    class?: string | undefined
    children?: JSX.Element
  }

const DialogContent = <T extends ValidComponent = "div">(
  props: PolymorphicProps<T, DialogContentProps<T>>
) => {
  const [, rest] = splitProps(props as DialogContentProps, ["class", "children"])
  return (
    <DialogPortal>
      <DialogOverlay />
      <DialogPrimitive.Content
        class={cn(
          "fixed left-1/2 top-1/2 z-50 grid max-h-[85vh] w-full max-w-lg -translate-x-1/2 -translate-y-1/2 overflow-y-auto",
          "bg-[#2d2d2d] border border-[#2a2a2a] rounded-[12px] shadow-[0_8px_32px_rgba(0,0,0,0.6)]",
          "text-[#cccccc] font-['IBM_Plex_Sans',-apple-system,BlinkMacSystemFont,sans-serif] text-[13px]",
          "duration-150 data-[expanded]:animate-in data-[closed]:animate-out data-[closed]:fade-out-0 data-[expanded]:fade-in-0 data-[closed]:zoom-out-98 data-[expanded]:zoom-in-98",
          props.class
        )}
        {...rest}
      >
        {props.children}
      </DialogPrimitive.Content>
    </DialogPortal>
  )
}

const DialogHeader: Component<ComponentProps<"div">> = (props) => {
  const [, rest] = splitProps(props, ["class"])
  return (
    <div
      class={cn(
        "flex items-center justify-between px-5 py-3 bg-[#333333] border-b border-[#2a2a2a] rounded-t-[12px] flex-shrink-0",
        props.class
      )}
      {...rest}
    />
  )
}

const DialogFooter: Component<ComponentProps<"div">> = (props) => {
  const [, rest] = splitProps(props, ["class"])
  return (
    <div
      class={cn(
        "flex items-center justify-end gap-2 px-5 py-3 bg-[#333333] border-t border-[#2a2a2a] rounded-b-[12px] flex-shrink-0",
        props.class
      )}
      {...rest}
    />
  )
}

type DialogTitleProps<T extends ValidComponent = "h2"> = DialogPrimitive.DialogTitleProps<T> & {
  class?: string | undefined
}

const DialogTitle = <T extends ValidComponent = "h2">(
  props: PolymorphicProps<T, DialogTitleProps<T>>
) => {
  const [, rest] = splitProps(props as DialogTitleProps, ["class"])
  return (
    <DialogPrimitive.Title
      class={cn("text-[14px] font-medium text-[#cccccc] leading-none", props.class)}
      {...rest}
    />
  )
}

type DialogDescriptionProps<T extends ValidComponent = "p"> =
  DialogPrimitive.DialogDescriptionProps<T> & {
    class?: string | undefined
  }

const DialogDescription = <T extends ValidComponent = "p">(
  props: PolymorphicProps<T, DialogDescriptionProps<T>>
) => {
  const [, rest] = splitProps(props as DialogDescriptionProps, ["class"])
  return (
    <DialogPrimitive.Description
      class={cn("text-[12px] text-[#808080]", props.class)}
      {...rest}
    />
  )
}

export {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription
}
