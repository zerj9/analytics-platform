"use client"
import React, { useState, useEffect, useRef } from 'react';
import { usePathname } from 'next/navigation'
import { Sidebar } from 'flowbite-react';

const sidebarTheme: CustomFlowbiteTheme['sidebar'] = {
  root: {
    inner: 'h-full overflow-y-auto overflow-x-hidden py-4 px-3 bg-sky-100'
  }
}

const createSidebarItems = (pages: string[], currentPath: string) => {
  const itemClassNames = "text-blue-950 hover:bg-blue-600 hover:text-white";
  const currentItemClassNames = "bg-blue-600 text-white hover:bg-blue-600 hover:text-white";
  let pageItems = []

  for (const page of pages) {
    const pageName = page.charAt(0).toUpperCase() + page.slice(1);
    if (page == "overview") {
      pageItems.push(<Sidebar.Item href="/" key={page} className={currentPath == "/" + page ? currentItemClassNames : itemClassNames}>{pageName}</Sidebar.Item>)
    } else {
      pageItems.push(<Sidebar.Item href={`/${page}`} key={page} className={currentPath == "/" + page ? currentItemClassNames : itemClassNames}>{pageName}</Sidebar.Item>)
    }
  }
  return (<>{pageItems}</>)
}

const MySidebar = () => {
  const pathname = usePathname()
  const [isOpen, setIsOpen] = useState(false); // controls visibility on mobile
  const sidebarRef = useRef(null); // ref for the sidebar for detecting outside clicks

  const toggleSidebar = () => {
    // Toggle only on mobile
    if (window.innerWidth < 768) {
      setIsOpen(!isOpen);
    }
  };

  // Detect all clicks on the document for mobile only
  useEffect(() => {
    function handleClickOutside(event) {
      if (window.innerWidth < 768 && sidebarRef.current && !sidebarRef.current.contains(event.target)) {
        setIsOpen(false); // Close the sidebar if click is outside
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  // Ensure sidebar is open by default on desktop
  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth >= 768) {
        setIsOpen(true); // Sidebar is open on desktop
      } else {
        setIsOpen(false); // Sidebar is controlled by state on mobile
      }
    };

    // Set initial state based on window size
    handleResize();

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);


  return (
    <>
      {/* Hamburger Menu Button - Shown only on mobile when sidebar is not open */}
      <div className={`fixed top-0 left-0 z-40 ${isOpen ? 'hidden' : 'block'}`}>
        <button
          className="p-2 text-gray-600 hover:text-gray-900 md:hidden"
          onClick={toggleSidebar}
          aria-label="Toggle sidebar"
        >
          {/* SVG for Hamburger Icon */}
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16m-7 6h7"/>
          </svg>
        </button>
      </div>

      {/* Sidebar */}
      <div
        ref={sidebarRef}
        className={`md:relative fixed inset-y-0 left-0  transform border-r-2 border-gray-300 ${isOpen ? 'translate-x-0' : '-translate-x-full'} md:translate-x-0 transition-transform duration-300 ease-in-out z-30 md:flex md:flex-shrink-0`}
        style={{ width: '256px' }} // Adjust width as needed
      >
        <Sidebar theme={sidebarTheme}>
          <Sidebar.Logo href="#" className="mt-5 text-blue-900">
            Analytics Platform
          </Sidebar.Logo>
          <Sidebar.Items className="mt-[50%]">
            <Sidebar.ItemGroup>
              {createSidebarItems(["overview", "data", "visualisations", "map"], pathname)}
            </Sidebar.ItemGroup>
          </Sidebar.Items>
        </Sidebar>
      </div>
    </>
  );
};

export default MySidebar;
