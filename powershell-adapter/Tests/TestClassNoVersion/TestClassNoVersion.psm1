[DscResource()]
class TestClassNoVersion
{
    [DscProperty(Key)]
    [string] $Name

    [void] Set()
    {
    }

    [bool] Test()
    {
        return $true
    }

    [TestClassNoVersion] Get()
    {
        return $this
    }
}