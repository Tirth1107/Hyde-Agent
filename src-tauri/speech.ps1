Add-Type -AssemblyName System.Speech

$recognizer = New-Object System.Speech.Recognition.SpeechRecognitionEngine
$recognizer.SetInputToDefaultAudioDevice()

$grammar = New-Object System.Speech.Recognition.DictationGrammar
$recognizer.LoadGrammar($grammar)

Write-Host "Listening..."
$result = $recognizer.Recognize()
if ($result -ne $null) {
    Write-Host "Recognized: $($result.Text)"
} else {
    Write-Host "Nothing recognized"
}
