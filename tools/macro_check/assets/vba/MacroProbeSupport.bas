Attribute VB_Name = "MacroProbeSupport"
Option Explicit

' 2026-04-22 CST: Added a minimal workbook-open probe so delivery checks can
' verify macro launch capability with one marker file and no business logic.
Public Sub RunMacroProbe()
    On Error GoTo ProbeFailed

    Dim markerPath As String
    Dim fileNumber As Integer

    markerPath = ThisWorkbook.Path & Application.PathSeparator & "macro_probe_signal.txt"
    fileNumber = FreeFile
    Open markerPath For Output As #fileNumber
    Print #fileNumber, "host_app=" & Application.Name
    Print #fileNumber, "workbook=" & ThisWorkbook.FullName
    Print #fileNumber, "opened_at=" & Format$(Now, "yyyy-mm-dd hh:nn:ss")
    Close #fileNumber

    Application.DisplayAlerts = False
    ThisWorkbook.Saved = True
    ThisWorkbook.Close SaveChanges:=False
    Exit Sub

ProbeFailed:
    On Error Resume Next
    fileNumber = FreeFile
    Open ThisWorkbook.Path & Application.PathSeparator & "macro_probe_error.txt" For Output As #fileNumber
    Print #fileNumber, "error_number=" & Err.Number
    Print #fileNumber, "error_description=" & Err.Description
    Close #fileNumber
End Sub
