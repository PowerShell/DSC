[DscResource()]
class TestClassResource
{
    [DscProperty(Key)]
    [string] $Prop1

    [void] Set()
    {
    }

    [bool] Test()
    {
        return $true
    }

    [TestClassResource] Get()
    {
        return $this
    }
}

function Hello-World()
{
    "Hello world from PSTestModule!"
}