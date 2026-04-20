Option Explicit

' 2026-04-21 CST: Rebuild the workbook runner around explicit source-sheet
' solving because approved multi-round debugging now needs sheet 1 and every
' result round to share one transparent, logged, cancelable execution path.
Private gCancelRequested As Boolean
Private gActiveExec As Object
Private gMacroLogPath As String

Private Const CURRENT_SHEET_NAME As String = "计算器"
Private Const RESULT_SHEET_PREFIX As String = "优化建议_第"
Private Const CURRENT_STATUS_CELL As String = "AH14"
Private Const RESULT_STATUS_CELL As String = "K7"
Private Const STATUS_WAITING As String = "正在计算中，请等待..."
Private Const STATUS_CANCELING As String = "正在取消，请稍候..."
Private Const STATUS_SUCCESS As String = "计算完成"
Private Const STATUS_FAILED As String = "计算失败，请查看日志"
Private Const STATUS_CANCELED As String = "本次计算已取消"
Private Const STATUS_TIMEOUT As String = "计算超时，已停止，请查看日志"
Private Const MAX_WAIT_SECONDS As Long = 300

Public Sub say_hello()
    RunBettingSolverForSheet CURRENT_SHEET_NAME
End Sub

Public Sub RunBettingSolverFromActiveSheet()
    RunBettingSolverForSheet ActiveSheet.Name
End Sub

Public Sub RequestCancel()
    gCancelRequested = True
    SafeAppendMacroLog "cancel_requested", "user clicked cancel button"
    On Error Resume Next
    SolverProgressForm.MarkCancelPending
    On Error GoTo 0
End Sub

Private Sub RunBettingSolverForSheet(ByVal sourceSheetName As String)
    On Error GoTo HandleError

    Dim workbookPath As String
    Dim workbookDir As String
    Dim solverPath As String
    Dim logDir As String
    Dim tempDir As String
    Dim tempOutputPath As String
    Dim tempResultWorkbook As Workbook
    Dim shell As Object
    Dim exec As Object
    Dim commandText As String
    Dim runStamp As String
    Dim startedAt As Date
    Dim elapsedSeconds As Long
    Dim stdoutText As String
    Dim stderrText As String
    Dim generatedSheetName As String

    workbookPath = ThisWorkbook.FullName
    workbookDir = ThisWorkbook.Path
    If Len(workbookDir) = 0 Then
        Err.Raise vbObjectError + 2000, "BettingSolverRunner", "请先保存工作簿后再测算。"
    End If

    solverPath = workbookDir & Application.PathSeparator & "betting_solver.exe"
    If Dir$(solverPath, vbNormal) = vbNullString Then
        Err.Raise vbObjectError + 2001, "BettingSolverRunner", "未找到 betting_solver.exe。"
    End If

    runStamp = Format(Now, "yyyymmdd_hhnnss")
    logDir = workbookDir & Application.PathSeparator & "logs"
    tempDir = workbookDir & Application.PathSeparator & ".betting_solver_tmp"
    EnsureDirectory logDir
    EnsureDirectory tempDir

    gMacroLogPath = logDir & Application.PathSeparator & "betting_macro_" & runStamp & ".log"
    AppendMacroLog "run_start", "source_sheet=" & sourceSheetName & " workbook=" & workbookPath

    If Not ThisWorkbook.Saved Then
        AppendMacroLog "workbook_save", "saving pending workbook edits before solving"
    End If
    ThisWorkbook.Save

    gCancelRequested = False
    tempOutputPath = tempDir & Application.PathSeparator & "solver_result_" & runStamp & ".xlsm"
    commandText = BuildSolveCommand(solverPath, workbookPath, tempOutputPath, logDir, sourceSheetName)

    UpdateStatusForSheet sourceSheetName, STATUS_WAITING
    ShowProgressForm STATUS_WAITING

    Set shell = CreateObject("WScript.Shell")
    Set exec = shell.Exec(commandText)
    Set gActiveExec = exec
    AppendMacroLog "solver_exec_started", commandText

    startedAt = Now
    Do While exec.Status = 0
        DoEvents
        elapsedSeconds = DateDiff("s", startedAt, Now)
        UpdateProgressForm STATUS_WAITING & vbCrLf & "已耗时 " & elapsedSeconds & " 秒"
        UpdateStatusForSheet sourceSheetName, STATUS_WAITING & "（" & elapsedSeconds & " 秒）"

        If gCancelRequested Then
            UpdateStatusForSheet sourceSheetName, STATUS_CANCELING
            exec.Terminate
            AppendMacroLog "solver_exec_canceled", "process terminated by user request"
            UpdateStatusForSheet sourceSheetName, STATUS_CANCELED
            GoTo SolverCanceled
        End If

        If elapsedSeconds >= MAX_WAIT_SECONDS Then
            exec.Terminate
            AppendMacroLog "solver_exec_timeout", "process exceeded max wait seconds"
            UpdateStatusForSheet sourceSheetName, STATUS_TIMEOUT
            GoTo SolverTimedOut
        End If
    Loop

    stdoutText = exec.StdOut.ReadAll
    stderrText = exec.StdErr.ReadAll
    AppendMacroLog "solver_exec_finished", "exit_code=" & exec.ExitCode
    If Len(stdoutText) > 0 Then
        AppendMacroLog "solver_stdout", stdoutText
    End If
    If Len(stderrText) > 0 Then
        AppendMacroLog "solver_stderr", stderrText
    End If

    If exec.ExitCode <> 0 Then
        UpdateStatusForSheet sourceSheetName, STATUS_FAILED
        GoTo SolverFailed
    End If

    AppendMacroLog "import_result_start", tempOutputPath
    Set tempResultWorkbook = Application.Workbooks.Open(tempOutputPath, ReadOnly:=True)
    generatedSheetName = ImportGeneratedRoundSheet(tempResultWorkbook, sourceSheetName)
    tempResultWorkbook.Close SaveChanges:=False
    Set tempResultWorkbook = Nothing
    AppendMacroLog "import_result_success", "imported_sheet=" & generatedSheetName

    On Error Resume Next
    Kill tempOutputPath
    On Error GoTo HandleError

    UpdateStatusForSheet sourceSheetName, STATUS_SUCCESS & " " & Format(Now, "hh:nn:ss")
    If Len(generatedSheetName) > 0 Then
        UpdateStatusForSheet generatedSheetName, "待再次测算"
    End If
    AppendMacroLog "run_success", "completed successfully"
    GoTo Cleanup

SolverFailed:
    AppendMacroLog "run_failed", "solver exit code was non-zero"
    GoTo Cleanup

SolverTimedOut:
    AppendMacroLog "run_timeout", "macro terminated solver after timeout"
    GoTo Cleanup

SolverCanceled:
    AppendMacroLog "run_canceled", "macro canceled the active solver run"
    GoTo Cleanup

HandleError:
    AppendMacroLog "macro_error", Err.Source & " | " & Err.Description
    UpdateStatusForSheet sourceSheetName, STATUS_FAILED

Cleanup:
    On Error Resume Next
    If Not tempResultWorkbook Is Nothing Then
        tempResultWorkbook.Close SaveChanges:=False
    End If
    HideProgressForm
    Application.StatusBar = False
    Set gActiveExec = Nothing
    Set shell = Nothing
    gCancelRequested = False
    On Error GoTo 0
End Sub

Private Function BuildSolveCommand( _
    ByVal solverPath As String, _
    ByVal workbookPath As String, _
    ByVal outputPath As String, _
    ByVal logDir As String, _
    ByVal sourceSheetName As String _
) As String
    BuildSolveCommand = Environ$("ComSpec") _
        & " /d /c set ""BETTING_SOLVER_LOG_DIR=" & logDir & """ && """ _
        & solverPath & """ solve """ & workbookPath & """ """ & outputPath _
        & """ --source-sheet """ & sourceSheetName & """"
End Function

Private Function ImportGeneratedRoundSheet( _
    ByVal sourceWorkbook As Workbook, _
    ByVal sourceSheetName As String _
) As String
    Dim copiedSheet As Worksheet
    Dim roundSheetName As String

    roundSheetName = FindGeneratedRoundSheetName(sourceWorkbook)
    If Len(roundSheetName) = 0 Then
        Err.Raise vbObjectError + 2002, "BettingSolverRunner", "结果工作簿中未找到新一轮结果页。"
    End If

    sourceWorkbook.Worksheets(roundSheetName).Copy After:=ThisWorkbook.Worksheets(ThisWorkbook.Worksheets.Count)
    Set copiedSheet = ThisWorkbook.Worksheets(ThisWorkbook.Worksheets.Count)
    copiedSheet.Name = roundSheetName

    On Error Resume Next
    If WorksheetExists(sourceSheetName) Then
        copiedSheet.Move After:=ThisWorkbook.Worksheets(sourceSheetName)
    End If
    On Error GoTo 0

    ThisWorkbook.Save
    ImportGeneratedRoundSheet = roundSheetName
End Function

Private Function FindGeneratedRoundSheetName(ByVal sourceWorkbook As Workbook) As String
    Dim sheet As Worksheet

    For Each sheet In sourceWorkbook.Worksheets
        If Left$(sheet.Name, Len(RESULT_SHEET_PREFIX)) = RESULT_SHEET_PREFIX Then
            FindGeneratedRoundSheetName = sheet.Name
            Exit Function
        End If
    Next sheet
End Function

Private Function WorksheetExists(ByVal sheetName As String) As Boolean
    Dim target As Worksheet

    On Error Resume Next
    Set target = ThisWorkbook.Worksheets(sheetName)
    WorksheetExists = Not target Is Nothing
    Set target = Nothing
    On Error GoTo 0
End Function

Private Sub EnsureDirectory(ByVal folderPath As String)
    If Len(Dir$(folderPath, vbDirectory)) = 0 Then
        MkDir folderPath
    End If
End Sub

Private Sub UpdateStatusForSheet(ByVal sheetName As String, ByVal message As String)
    On Error Resume Next
    If sheetName = CURRENT_SHEET_NAME Then
        ThisWorkbook.Worksheets(CURRENT_SHEET_NAME).Range(CURRENT_STATUS_CELL).Value = message
    ElseIf WorksheetExists(sheetName) Then
        ThisWorkbook.Worksheets(sheetName).Range(RESULT_STATUS_CELL).Value = message
    End If
    Application.StatusBar = message
    On Error GoTo 0
End Sub

Private Sub ShowProgressForm(ByVal message As String)
    On Error Resume Next
    Load SolverProgressForm
    SolverProgressForm.SetStatusText message
    SolverProgressForm.Show vbModeless
    On Error GoTo 0
End Sub

Private Sub UpdateProgressForm(ByVal message As String)
    On Error Resume Next
    SolverProgressForm.SetStatusText message
    On Error GoTo 0
End Sub

Private Sub HideProgressForm()
    On Error Resume Next
    Unload SolverProgressForm
    On Error GoTo 0
End Sub

Private Sub AppendMacroLog(ByVal eventName As String, ByVal message As String)
    Dim fileNumber As Integer
    fileNumber = FreeFile
    Open gMacroLogPath For Append As #fileNumber
    Print #fileNumber, "[" & Format(Now, "yyyy-mm-dd hh:nn:ss") & "] " & eventName & " " & message
    Close #fileNumber
End Sub

Private Sub SafeAppendMacroLog(ByVal eventName As String, ByVal message As String)
    On Error Resume Next
    AppendMacroLog eventName, message
    On Error GoTo 0
End Sub
