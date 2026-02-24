; NetNinja NSIS Installer Hooks
; These macros are called by the Tauri NSIS installer at specific points
; during installation and uninstallation.

; Called before files are copied during installation.
; Stops the NetNinja Windows Service if it is already running from a previous
; installation, so the installer can overwrite netninja-service.exe without
; hitting a file-lock error.
!macro NSIS_HOOK_PREINSTALL
  ; Stop the running service via SCM (ignore errors if not installed)
  nsExec::ExecToLog 'sc stop netninja-scheduler'

  ; Wait for the service to fully stop
  Sleep 3000

  ; Force-kill the process if it's still running
  nsExec::ExecToLog 'taskkill /F /IM netninja-service.exe'
!macroend

; Called after all files are copied, registry keys set, and shortcuts created.
; Installs and starts the NetNinja Windows Service.
!macro NSIS_HOOK_POSTINSTALL
  ; Install the Windows service (registers with SCM, sets auto-start)
  ExecWait '"$INSTDIR\netninja-service.exe" install'

  ; Grant Users modify permissions on the data directory so the desktop app
  ; (running as a regular user) can write to the SQLite database
  nsExec::ExecToLog 'icacls "$COMMONPROGRAMDATA\NetNinja" /grant Users:(OI)(CI)M /T /Q'

  ; Start the service immediately
  ExecWait '"$INSTDIR\netninja-service.exe" start'
!macroend

; Called before any files, registry keys, or shortcuts are removed.
; Stops and uninstalls the NetNinja Windows Service, then removes all data.
;
; Uses sc.exe (always available on Windows) instead of the service binary
; because the running service process may hold a lock on its own exe,
; causing ExecWait on that binary to fail.
!macro NSIS_HOOK_PREUNINSTALL
  ; Stop the running service via SCM
  nsExec::ExecToLog 'sc stop netninja-scheduler'

  ; Wait for the service to fully stop before deleting
  ; sc delete will fail if the service is still in STOP_PENDING state
  Sleep 3000

  ; Delete the service from SCM
  nsExec::ExecToLog 'sc delete netninja-scheduler'

  ; Force-kill the process if it's still running (handles edge cases)
  nsExec::ExecToLog 'taskkill /F /IM netninja-service.exe'

  ; Delete the ProgramData directory and all contents
  RMDir /r "$COMMONPROGRAMDATA\NetNinja"
!macroend
