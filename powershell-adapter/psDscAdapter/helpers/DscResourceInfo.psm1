Set-StrictMode -Version latest
$ErrorActionPreference = 'Stop'

$AddDscResourceInfoTypeScript = @"
//-----------------------------------------------------------------------
// <copyright file="DscResourceInfo.cs" company="Microsoft Corporation">
//     Copyright (C) 2013 Microsoft Corporation
// </copyright>
//-----------------------------------------------------------------------

using System.Collections.Generic;
using System;
using System.Management.Automation;
using System.IO;

namespace Microsoft.PowerShell.DesiredStateConfiguration
{
    /// <summary>
    /// Enumerated values for DSC resource implementation type
    /// </summary>
    public enum ImplementedAsType
    {
        /// <summary>
        /// DSC resource implementation type not known
        /// </summary>
        None = 0,
        
        /// <summary>
        /// DSC resource is implemented using PowerShell module
        /// </summary>
        PowerShell = 1,

        /// <summary>
        /// DSC resource is implemented using a CIM provider
        /// </summary>
        Binary = 2,

        /// <summary>
        /// DSC resource is a composite and implemented using configuration keyword
        /// </summary>
        Composite = 3
    }

    /// <summary>
    /// Contains a DSC resource information
    /// </summary>
    public sealed class DscResourceInfo
    {
        /// <summary>
        /// Initializes a new instance of the DscResourceInfo class
        /// </summary>
        public DscResourceInfo()
        {
            this.Properties = new List<DscResourcePropertyInfo>();
        }

        /// <summary>
        /// Gets or sets resource type name
        /// </summary>
        public string ResourceType { get; set; }

        /// <summary>
        /// Gets or sets Name of the resource. This name is used to access the resource
        /// </summary>
        public string Name { get; set; }

        /// <summary>
        /// Gets or sets friendly name defined for the resource
        /// </summary>
        public string FriendlyName { get; set; }

        /// <summary>
        /// Gets or sets module which implements the resource. This could point to parent module, if the DSC resource is implemented 
        /// by one of nested modules.
        /// </summary>
        public PSModuleInfo Module { get; set; }

        /// <summary>
        /// Gets name of the module which implements the resource. 
        /// </summary>
        public string ModuleName
        {
            get
            {
                if (this.Module == null) return null;
                return this.Module.Name;
            }
        }

        /// <summary>
        /// Gets version of the module which implements the resource. 
        /// </summary>
        public Version Version
        {
            get
            {
                if (this.Module == null) return null;
                return this.Module.Version;
            }
        }

        /// <summary>
        /// Gets or sets of the file which implements the resource. For the reosurces which are defined using 
        /// MOF file, this will be path to a module which resides in the same folder where schema.mof file is present.
        /// For composite resources, this will be the module which implements the resource
        /// </summary>
        public string Path { get; set; }

        /// <summary>
        /// Gets or sets parent folder, where the resource is defined
        /// It is the folder containing either the implementing module(=Path) or folder containing ".schema.mof". 
        /// For native providers, Path will be null and only ParentPath will be present.
        /// </summary>
        public string ParentPath { get; set; }

        /// <summary>
        /// Gets or sets a value which indicate how DSC resource is implemented
        /// </summary>
        public ImplementedAsType ImplementedAs { get; set; }

        /// <summary>
        /// Gets or sets company which owns this resource
        /// </summary>
        public string CompanyName { get; set; }

        /// <summary>
        /// Gets or sets properties of the resource
        /// </summary>
        public List<DscResourcePropertyInfo> Properties { get; private set; }

        /// <summary>
        /// Updates properties of the resource
        /// </summary>
        /// <param name="properties">Updated properties</param>
        public void UpdateProperties(List<DscResourcePropertyInfo> properties)
        {
            this.Properties = properties;
        }
    }

    /// <summary>
    /// Contains a DSC resource property information
    /// </summary>
    public sealed class DscResourcePropertyInfo
    {
        /// <summary>
        /// Initializes a new instance of the DscResourcePropertyInfo class
        /// </summary>
        public DscResourcePropertyInfo()
        {
            this.Values = new List<string>();
        }
        
        /// <summary>
        /// Gets or sets name of the property
        /// </summary>
        public string Name { get; set; }

        /// <summary>
        /// Gets or sets type of the property
        /// </summary>
        public string PropertyType { get; set; }

        /// <summary>
        /// Gets or sets a value indicating whether the property is mandatory or not
        /// </summary>
        public bool IsMandatory { get; set; }

        /// <summary>
        /// Gets Values for a resource property
        /// </summary>
        public List<string> Values { get; private set; }
    }
}
"@

if(-not ([System.Management.Automation.PSTypeName]'Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo').Type) {
    Add-Type -TypeDefinition $AddDscResourceInfoTypeScript
}
