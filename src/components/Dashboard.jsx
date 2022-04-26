import React from 'react';
import PropTypes from 'prop-types';
import { Link } from 'react-router-dom';

const Dashboard = ({version, currentUser}) => {
   return <>
                 <header>
                   <h1>NEAR Challenge #3 - Social Bounty Hunt - {version}</h1>
                 </header>
                  <p>
                      This app demonstrates how to mint nfts in with the NEAR blockchain. While minting
                      the app will ask you to deposite 0.1 NEAR but it acutally only uses roughly 0.01 NEAR.
                      The remaining NEAR that is not used gets refunded on completing the smart contrat call.
                  </p>
                  <p>
                      The basic idea was to create a DeviantArt for the NEAR chain. So everyone can mint their
                      own artworks and through the marketplace they can then buy physical and digital products
                      from the minted NFTs. This includes, but is not limited to, buying rights to use the images
                      on their websites. The information, who has rights could be stored in the metadata of the NFTs.
                      This site yould then provide tools to check whether a specific webpage has the rights to use
                      the artwork.
                  </p>
                 <h5>Head over to <Link className="menu-item" to="/collection">your collection</Link> to mint 
                     you first artwork or go to <Link className="menu-item" to="/marketplace">the marketplace</Link> to
                     discover what others created.
                 </h5>
             </>
}

Dashboard.propTypes = {
  version: PropTypes.string.isRequired,
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  })
};

export default Dashboard;