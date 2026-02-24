import { type Component, createSignal } from 'solid-js';
import { EmailSettings as EmailSettingsContent } from '~/components/settings/EmailSettings';

export const EmailSettings: Component = () => {
  const [addTrigger, setAddTrigger] = createSignal(0);

  const handleAdd = () => {
    setAddTrigger(prev => prev + 1);
  };

  return (
    <div class="flex h-full">
      <EmailSettingsContent addTrigger={addTrigger()} onAdd={handleAdd} />
    </div>
  );
};
