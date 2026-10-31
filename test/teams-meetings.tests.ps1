. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'teams-meetings')

Test 'meeting window title without the Teams suffix' {
    Assert-Equal 'Contoso sprint review' (Select-MeetingTitle @('Chat | Microsoft Teams', 'Contoso sprint review | Microsoft Teams'))
}

Test 'sections of the main window are not meetings' {
    Assert-Equal $null (Select-MeetingTitle @('Calendar | Microsoft Teams', 'Activity | Microsoft Teams', 'Microsoft Teams'))
}

Test 'no windows, no meeting' {
    Assert-Equal $null (Select-MeetingTitle @())
}

Complete-Tests
