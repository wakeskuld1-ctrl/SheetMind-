Option Explicit

' 2026-04-20 CST: Add a modeless progress surface because the user explicitly
' asked for a visible “calculating” state and a dedicated cancel action instead
' of a silent blocking button flow.
Private Sub UserForm_Initialize()
    ' 2026-04-20 CST: Restore localized captions at runtime because the
    ' PowerShell VBA builder stays ASCII-only to avoid Windows PowerShell 5
    ' UTF-8 parser regressions during delivery asset generation.
    Me.Caption = "正在计算中"
    Me.lblStatus.Caption = "正在计算中，请等待..."
    Me.lblHint.Caption = "如需停止，请点击取消"
    Me.btnCancel.Caption = "取消"
End Sub

Public Sub SetStatusText(ByVal message As String)
    Me.lblStatus.Caption = message
End Sub

Public Sub MarkCancelPending()
    Me.btnCancel.Caption = "正在取消..."
    Me.btnCancel.Enabled = False
    Me.lblHint.Caption = "正在停止本次计算，请稍候..."
End Sub

Private Sub btnCancel_Click()
    RequestCancel
End Sub
